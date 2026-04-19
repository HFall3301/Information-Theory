mod algorithms;

use algorithms::{RsaParams, build_rsa_params, decrypt_file, encrypt_file, read_cipher_blocks};
use eframe::egui;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([980.0, 680.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RSA lab03 - Kuzhik Daniil, 451003",
        options,
        Box::new(|_cc| Ok(Box::new(CryptoApp::default()))),
    )
}

struct CryptoApp {
    p_input: String,
    q_input: String,
    d_input: String,

    input_path: String,
    output_path: String,

    status_message: String,
    cipher_blocks: Vec<u16>,
    last_action: String,

    current_offset: u64,
    chunk_size: u64,
}

impl Default for CryptoApp {
    fn default() -> Self {
        Self {
            p_input: "251".to_string(),
            q_input: "241".to_string(),
            d_input: "17".to_string(),
            input_path: "input.bin".to_string(),
            output_path: "output.bin".to_string(),
            status_message: "Готово к работе".to_string(),
            cipher_blocks: Vec::new(),
            last_action: "-".to_string(),
            current_offset: 0,
            chunk_size: 256,
        }
    }
}

fn format_binary_view(data: &[u8]) -> String {
    if data.is_empty() {
        return "(пусто)".to_string();
    }

    let mut result = String::new();
    for (i, &byte) in data.iter().enumerate() {
        result.push_str(&format!("{:08b} ", byte));

        if (i + 1) % 4 == 0 && i != data.len() - 1 {
            result.push('\n');
        }
    }
    result
}

fn format_cipher_blocks_decimal(blocks: &[u16]) -> String {
    if blocks.is_empty() {
        return "(пусто)".to_string();
    }

    let mut out = String::new();
    for (i, block) in blocks.iter().enumerate() {
        out.push_str(&block.to_string());
        if i + 1 != blocks.len() {
            if (i + 1) % 12 == 0 {
                out.push('\n');
            } else {
                out.push(' ');
            }
        }
    }
    out
}

impl CryptoApp {
    fn sanitize_numeric_input(value: &mut String) {
        value.retain(|c| c.is_ascii_digit());
    }

    fn parse_value(name: &str, value: &str) -> Result<u64, String> {
        if value.trim().is_empty() {
            return Err(format!("Поле {name} пустое."));
        }

        value
            .trim()
            .parse::<u64>()
            .map_err(|_| format!("Поле {name} должно быть целым положительным числом."))
    }

    fn rsa_params(&self) -> Result<RsaParams, String> {
        let p = Self::parse_value("p", &self.p_input)?;
        let q = Self::parse_value("q", &self.q_input)?;
        let d = Self::parse_value("d", &self.d_input)?;
        build_rsa_params(p, q, d)
    }

    fn validate_paths(&self) -> Result<(), String> {
        if self.input_path.trim().is_empty() {
            return Err("Не указан входной файл.".to_string());
        }
        if self.output_path.trim().is_empty() {
            return Err("Не указан выходной файл.".to_string());
        }
        if self.input_path == self.output_path {
            return Err("Входной и выходной файл должны быть разными.".to_string());
        }
        Ok(())
    }

    fn encrypt_action(&mut self) {
        if let Err(e) = self.validate_paths() {
            self.status_message = e;
            return;
        }

        let params = match self.rsa_params() {
            Ok(params) => params,
            Err(e) => {
                self.status_message = e;
                return;
            }
        };

        match encrypt_file(&self.input_path, &self.output_path, &params) {
            Ok(()) => match read_cipher_blocks(&self.output_path) {
                Ok(blocks) => {
                    self.cipher_blocks = blocks;
                    self.current_offset = 0;
                    self.clamp_offset();
                    self.last_action = "Шифрование".to_string();
                    self.status_message = format!(
                        "Шифрование успешно завершено. Сформировано {} блоков по 16 бит.",
                        self.cipher_blocks.len()
                    );
                }
                Err(e) => {
                    self.status_message = format!(
                        "Файл зашифрован, но не удалось прочитать блоки для вывода: {e}"
                    );
                }
            },
            Err(e) => {
                self.status_message = format!("Ошибка шифрования: {e}");
            }
        }
    }

    fn decrypt_action(&mut self) {
        if let Err(e) = self.validate_paths() {
            self.status_message = e;
            return;
        }

        let params = match self.rsa_params() {
            Ok(params) => params,
            Err(e) => {
                self.status_message = e;
                return;
            }
        };

        match read_cipher_blocks(&self.input_path) {
            Ok(blocks) => {
                self.cipher_blocks = blocks;
                self.current_offset = 0;
                self.clamp_offset();
            }
            Err(e) => {
                self.status_message = format!("Ошибка чтения шифротекста: {e}");
                return;
            }
        }

        match decrypt_file(&self.input_path, &self.output_path, &params) {
            Ok(()) => {
                self.last_action = "Расшифрование".to_string();
                self.status_message = "Расшифрование успешно завершено.".to_string();
            }
            Err(e) => {
                self.status_message = format!("Ошибка расшифрования: {e}");
            }
        }
    }

    fn data_byte_len(&self) -> u64 {
        self.cipher_blocks.len() as u64
    }

    fn plaintext_source_path(&self) -> Option<&str> {
        match self.last_action.as_str() {
            "Шифрование" => Some(self.input_path.as_str()),
            "Расшифрование" => Some(self.output_path.as_str()),
            _ => None,
        }
    }

    fn read_chunk(path: &str, offset: u64, chunk_size: u64) -> Vec<u8> {
        if !std::path::Path::new(path).exists() {
            return Vec::new();
        }

        let mut buffer = vec![0u8; chunk_size as usize];
        if let Ok(mut file) = File::open(path) {
            let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
            let actual_offset = offset.min(file_size);

            if file.seek(SeekFrom::Start(actual_offset)).is_ok() {
                let bytes_read = file.read(&mut buffer).unwrap_or(0);
                buffer.truncate(bytes_read);
                return buffer;
            }
        }
        Vec::new()
    }

    fn can_display_chunk(&self, offset: u64, data_len: u64) -> bool {
        if data_len == 0 {
            return false;
        }
        offset < data_len
    }

    fn clamp_offset(&mut self) {
        let data_len = self.data_byte_len();
        if data_len == 0 {
            self.current_offset = 0;
            return;
        }

        let max_offset = data_len.saturating_sub(self.chunk_size);
        if self.current_offset > max_offset {
            self.current_offset = max_offset;
        }
    }

    fn update_offset_and_views(&mut self, new_offset: u64) {
        let data_len = self.data_byte_len();
        if self.can_display_chunk(new_offset, data_len) {
            self.current_offset = new_offset;
        } else if data_len > 0 {
            let last_offset = data_len.saturating_sub(self.chunk_size);
            if last_offset != self.current_offset {
                self.current_offset = last_offset;
            }
        }
    }
}

impl eframe::App for CryptoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        egui::TopBottomPanel::top("top_panel").show(&ctx, |ui| {
            ui.heading("ЛР3: RSA (побайтовое шифрование, 16-битный шифротекст)");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("p:");
                if ui.text_edit_singleline(&mut self.p_input).changed() {
                    Self::sanitize_numeric_input(&mut self.p_input);
                }

                ui.label("q:");
                if ui.text_edit_singleline(&mut self.q_input).changed() {
                    Self::sanitize_numeric_input(&mut self.q_input);
                }

                ui.label("d (закрытый ключ):");
                if ui.text_edit_singleline(&mut self.d_input).changed() {
                    Self::sanitize_numeric_input(&mut self.d_input);
                }
            });

            match self.rsa_params() {
                Ok(params) => {
                    ui.label(format!(
                        "Параметры: r = {} ; phi(r) = {} ; e (открытый ключ) = {}",
                        params.r, params.phi, params.e
                    ));
                    ui.label(format!(
                        "Проверка диапазона модуля: 255 < r <= 65535 (сейчас r = {})",
                        params.r
                    ));
                    ui.label(format!("Используемые простые: p = {}, q = {}", params.p, params.q));
                }
                Err(e) => {
                    ui.colored_label(egui::Color32::from_rgb(210, 60, 60), format!("Проверка параметров: {e}"));
                }
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Входной файл:");
                ui.text_edit_singleline(&mut self.input_path);
                if ui.button("Выбрать").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.input_path = path.display().to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Выходной файл:");
                ui.text_edit_singleline(&mut self.output_path);
                if ui.button("Сохранить как").clicked() {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        self.output_path = path.display().to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Зашифровать").clicked() {
                    self.encrypt_action();
                }
                if ui.button("Расшифровать").clicked() {
                    self.decrypt_action();
                }
                ui.label(format!("Последняя операция: {}", self.last_action));
            });

            ui.label(format!("Статус: {}", self.status_message));
        });

        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.heading("Исходный текст и шифротекст (16-битные блоки, десятичные)");
            ui.label(format!("Всего блоков: {}", self.cipher_blocks.len()));
            ui.separator();

            let data_len = self.data_byte_len();
            let avail = data_len.saturating_sub(self.current_offset);
            let take = self.chunk_size.min(avail);

            let (orig_text, cipher_text) = if data_len == 0 {
                (
                    "(нет данных — выполните шифрование или расшифрование)".to_string(),
                    "(нет данных)".to_string(),
                )
            } else {
                let take_usize = take as usize;
                let start = self.current_offset as usize;
                let cipher_slice = &self.cipher_blocks[start..start + take_usize];
                let cipher_str = format_cipher_blocks_decimal(cipher_slice);

                let orig_str = match self.plaintext_source_path() {
                    Some(path) => {
                        let raw = Self::read_chunk(path, self.current_offset, take);
                        format_binary_view(&raw)
                    }
                    None => "(выполните шифрование или расшифрование для сравнения с исходником)"
                        .to_string(),
                };

                (orig_str, cipher_str)
            };

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.columns(2, |columns| {
                        columns[0].heading("Исходные байты");
                        columns[0].label(egui::RichText::new(orig_text).monospace());

                        columns[1].heading("Шифротекст (десятичные блоки)");
                        columns[1].label(egui::RichText::new(cipher_text).monospace());
                    });
                });

            ui.separator();

            ui.horizontal(|ui| {
                let max_offset = data_len.saturating_sub(self.chunk_size);

                if ui.button("назад").clicked() {
                    let new_offset = self.current_offset.saturating_sub(self.chunk_size);
                    self.update_offset_and_views(new_offset);
                }

                if data_len > self.chunk_size {
                    let mut offset_f64 = self.current_offset as f64;
                    if ui
                        .add(
                            egui::Slider::new(&mut offset_f64, 0.0..=max_offset as f64)
                                .text("Смещение (байт)"),
                        )
                        .changed()
                    {
                        self.update_offset_and_views(offset_f64 as u64);
                    }
                } else if data_len > 0 {
                    ui.label(format!("Размер данных: {} байт", data_len));
                }

                if ui.button("вперёд").clicked() {
                    let new_offset = (self.current_offset + self.chunk_size).min(max_offset);
                    self.update_offset_and_views(new_offset);
                }
            });
        });
    }
}


