mod algorithms;

use eframe::egui;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use crate::algorithms::{Cipher, LFSR, process_file};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Kuzhik Daniil, 451003",
        options,
        Box::new(|_cc| Ok(Box::new(CryptoApp::default()))),
    )
}

struct CryptoApp {
    seed_input: String,
    input_path: String,
    output_path: String,
    key_path: String,

    file_size: u64,
    current_offset: u64,
    chunk_size: u64,

    orig_view: Vec<u8>,
    key_view: Vec<u8>,
    out_view: Vec<u8>,

    is_processing: Arc<AtomicBool>,
    last_processing_state: bool,
    progress: Arc<AtomicU64>,
    status_message: String,
    last_update: std::time::Instant,
}

impl Default for CryptoApp {
    fn default() -> Self {
        Self {
            seed_input: "1".repeat(30),
            input_path: "input.txt".to_string(),
            output_path: "output.bin".to_string(),
            key_path: ".temp_key.bin".to_string(),
            file_size: 0,
            current_offset: 0,
            chunk_size: 256,
            orig_view: Vec::new(),
            key_view: Vec::new(),
            out_view: Vec::new(),
            is_processing: Arc::new(AtomicBool::new(false)),
            last_processing_state: false,
            progress: Arc::new(AtomicU64::new(0)),
            status_message: "Готов к работе".to_string(),
            last_update: std::time::Instant::now(),
        }
    }
}

impl Drop for CryptoApp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.key_path);
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

impl CryptoApp {
    fn read_chunk(&self, path: &str) -> Vec<u8> {
        if !std::path::Path::new(path).exists() {
            return Vec::new();
        }

        let mut buffer = vec![0u8; self.chunk_size as usize];
        if let Ok(mut file) = File::open(path) {
            let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
            let actual_offset = self.current_offset.min(file_size);

            if file.seek(SeekFrom::Start(actual_offset)).is_ok() {
                let bytes_read = file.read(&mut buffer).unwrap_or(0);
                buffer.truncate(bytes_read);
                return buffer;
            }
        }
        Vec::new()
    }

    fn update_views(&mut self) {
        self.orig_view = self.read_chunk(&self.input_path);
        self.key_view = self.read_chunk(&self.key_path);
        self.out_view = self.read_chunk(&self.output_path);
    }

    fn start_processing(&mut self) {
        if self.is_processing.load(Ordering::SeqCst) { return; }

        let meta = match std::fs::metadata(&self.input_path) {
            Ok(m) => m,
            Err(_) => {
                self.status_message = "Ошибка: Файл не найден".to_string();
                return;
            }
        };

        if meta.len() == 0 {
            self.status_message = "Ошибка: Входной файл пуст".to_string();
            return;
        }

        self.file_size = meta.len();

        let seed = u64::from_str_radix(&self.seed_input, 2).unwrap_or(0);
        let mut lfsr = LFSR {
            register: seed,
            ..Default::default()
        };

        let is_processing = self.is_processing.clone();
        let progress = self.progress.clone();
        let in_p = self.input_path.clone();
        let out_p = self.output_path.clone();
        let key_p = self.key_path.clone();

        let _ = std::fs::File::create(&key_p);

        is_processing.store(true, Ordering::SeqCst);
        progress.store(0, Ordering::SeqCst);
        self.status_message = if meta.len() < self.chunk_size {
            format!("Шифрование малого файла ({} байт)...", meta.len())
        } else {
            "Шифрование...".to_string()
        };

        thread::spawn(move || {
            let result = process_file(in_p, out_p, key_p, &mut lfsr, progress);

            if let Err(e) = result {
                eprintln!("Ошибка при обработке: {}", e);
            }

            is_processing.store(false, Ordering::SeqCst);
        });
    }
    fn can_display_chunk(&self, offset: u64, file_size: u64) -> bool {
        if file_size == 0 {
            return false;
        }
        offset < file_size
    }

    fn update_offset_and_views(&mut self, new_offset: u64) {
        if self.can_display_chunk(new_offset, self.file_size) {
            self.current_offset = new_offset;
            self.update_views();
        } else if self.file_size > 0 {
            let last_offset = self.file_size.saturating_sub(self.chunk_size);
            if last_offset != self.current_offset {
                self.current_offset = last_offset;
                self.update_views();
            }
        }
    }
}
impl eframe::App for CryptoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let current_processing = self.is_processing.load(Ordering::SeqCst);

        if self.last_processing_state && !current_processing {
            self.file_size = std::fs::metadata(&self.input_path)
                .map(|m| m.len())
                .unwrap_or(0);

            if self.current_offset >= self.file_size && self.file_size > 0 {
                self.current_offset = 0;
            }

            self.update_views();

            let progress_val = self.progress.load(Ordering::SeqCst);
            if progress_val >= self.file_size && self.file_size > 0 {
                self.status_message = "✅ Готово!".to_string();
            } else if self.file_size == 0 {
                self.status_message = "❌ Файл пуст".to_string();
            }

            self.last_update = std::time::Instant::now();
        } else if !current_processing && self.last_update.elapsed() > std::time::Duration::from_millis(100) {
            self.update_views();
            self.last_update = std::time::Instant::now();
        }

        self.last_processing_state = current_processing;

        if current_processing {
            ctx.request_repaint();
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("LFSR Шифратор/Дешифратор");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Начальное состояние регистра (30 бит):");
                let response = ui.text_edit_singleline(&mut self.seed_input);
                if response.changed() {
                    self.seed_input.retain(|c| c == '0' || c == '1');
                    if self.seed_input.len() > 30 {
                        self.seed_input.truncate(30);
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Входной файл:");
                ui.text_edit_singleline(&mut self.input_path);
                if ui.button("📁 Выбрать").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.input_path = path.display().to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Выходной файл:");
                ui.text_edit_singleline(&mut self.output_path);
                if ui.button("Выбрать куда сохранить").clicked() {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        self.output_path = path.display().to_string();
                    }
                }
            });

            if ui.button("Зашифровать / Расшифровать").clicked() {
                self.start_processing();
            }
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let max_offset = self.file_size.saturating_sub(self.chunk_size);

                if ui.button("назад").clicked() {
                    let new_offset = self.current_offset.saturating_sub(self.chunk_size);
                    self.update_offset_and_views(new_offset);
                }

                if self.file_size > self.chunk_size {
                    let mut offset_f64 = self.current_offset as f64;
                    if ui.add(egui::Slider::new(&mut offset_f64, 0.0..=max_offset as f64)
                        .text("Смещение (байт)")).changed() {
                        self.update_offset_and_views(offset_f64 as u64);
                    }
                } else {
                    ui.label(format!("Размер файла: {} байт", self.file_size));
                }

                if ui.button("вперед").clicked() {
                    let new_offset = (self.current_offset + self.chunk_size).min(max_offset);
                    self.update_offset_and_views(new_offset);
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.is_processing.load(Ordering::SeqCst) {
                        let p = self.progress.load(Ordering::SeqCst);
                        ui.spinner();
                        ui.label(format!("Обработано: {} / {} байт", p, self.file_size));
                    } else {
                        if self.file_size > 0 && self.progress.load(Ordering::SeqCst) >= self.file_size {
                            ui.label("✅ Готово");
                        } else {
                            ui.label("💤 Ожидание");
                        }
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.columns(3, |columns| {
                    columns[0].heading("Исходный файл");
                    let orig_text = format_binary_view(&self.orig_view);
                    columns[0].label(egui::RichText::new(orig_text).monospace());

                    columns[1].heading("Сгенерированный ключ");
                    let key_text = format_binary_view(&self.key_view);
                    columns[1].label(egui::RichText::new(key_text).monospace());

                    columns[2].heading("Результат");
                    let out_text = format_binary_view(&self.out_view);
                    columns[2].label(egui::RichText::new(out_text).monospace());
                });
            });
        });
    }
}