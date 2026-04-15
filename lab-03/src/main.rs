mod algorithms;

use algorithms::{RsaParams, build_rsa_params, decrypt_file, encrypt_file, read_cipher_blocks};
use eframe::egui;

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
        }
    }
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

    fn blocks_as_decimal_text(&self, max_items: usize) -> String {
        if self.cipher_blocks.is_empty() {
            return "(нет данных)".to_string();
        }

        let take_n = self.cipher_blocks.len().min(max_items);
        let mut out = String::new();

        for (i, block) in self.cipher_blocks.iter().take(take_n).enumerate() {
            out.push_str(&block.to_string());
            if i + 1 != take_n {
                if (i + 1) % 12 == 0 {
                    out.push('\n');
                } else {
                    out.push(' ');
                }
            }
        }

        if self.cipher_blocks.len() > max_items {
            out.push_str(&format!(
                "\n... показано {} из {} блоков",
                max_items,
                self.cipher_blocks.len()
            ));
        }

        out
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
            ui.heading("Шифротекст в десятичной системе (16-битные блоки)");
            ui.label(format!("Всего блоков: {}", self.cipher_blocks.len()));
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let text = self.blocks_as_decimal_text(1200);
                ui.label(egui::RichText::new(text).monospace());
            });
        });
    }
}


