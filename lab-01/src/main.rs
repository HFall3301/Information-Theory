use eframe::egui;
use egui::{Align, Layout};
use rfd::FileDialog;
use std::fs;

mod algorithms;
mod alphabet;

#[derive(PartialEq)]
enum EncryptionAlgorithm {
    DecimationEnglish,
    VigenereRussian,
}

struct CipherApp {
    key_input: String,
    input_text: String,
    output_text: String,
    selected_algorithm: EncryptionAlgorithm,
    error_message: Option<String>,
}

impl Default for CipherApp {
    fn default() -> Self {
        Self {
            key_input: String::new(),
            input_text: String::new(),
            output_text: String::new(),
            selected_algorithm: EncryptionAlgorithm::DecimationEnglish,
            error_message: None,
        }
    }
}

impl CipherApp {
    fn clear_fields(&mut self) {
        self.key_input.clear();
        self.input_text.clear();
        self.output_text.clear();
    }

    fn encrypt_action(&mut self) {
        if self.key_input.trim().is_empty() {
            self.error_message = Some("Введите ключ.".into());
            return;
        }

        if self.input_text.trim().is_empty() {
            self.error_message = Some("Введите текст.".into());
            return;
        }

        let result = match self.selected_algorithm {
            EncryptionAlgorithm::DecimationEnglish => {
                algorithms::encrypt_decimation(&self.input_text, &self.key_input)
            }
            EncryptionAlgorithm::VigenereRussian => {
                algorithms::encrypt_vigenere_ru(&self.input_text, &self.key_input)
            }
        };

        match result {
            Ok(text) => self.output_text = text,
            Err(err) => self.error_message = Some(err),
        }
    }

    fn decrypt_action(&mut self) {
        if self.key_input.trim().is_empty() {
            self.error_message = Some("Введите ключ.".into());
            return;
        }

        if self.input_text.trim().is_empty() {
            self.error_message = Some("Введите текст.".into());
            return;
        }

        let result = match self.selected_algorithm {
            EncryptionAlgorithm::DecimationEnglish => {
                algorithms::decrypt_decimation(&self.input_text, &self.key_input)
            }
            EncryptionAlgorithm::VigenereRussian => {
                algorithms::decrypt_vigenere_ru(&self.input_text, &self.key_input)
            }
        };

        match result {
            Ok(text) => self.output_text = text,
            Err(err) => self.error_message = Some(err),
        }
    }
}

impl eframe::App for CipherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ===== MENU =====
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Файл", |ui| {
                    if ui.button("Открыть файл").clicked() {
                        if let Some(path) = FileDialog::new().pick_file() {
                            match fs::read_to_string(path) {
                                Ok(content) => self.input_text = content,
                                Err(_) => self.error_message =
                                    Some("Ошибка чтения файла.".into()),
                            }
                        }
                        ui.close();
                    }

                    if ui.button("Сохранить результат").clicked() {
                        if self.output_text.is_empty() {
                            self.error_message =
                                Some("Нет данных для сохранения.".into());
                        } else if let Some(path) =
                            FileDialog::new().save_file()
                        {
                            if fs::write(path, &self.output_text).is_err() {
                                self.error_message =
                                    Some("Ошибка записи файла.".into());
                            }
                        }
                        ui.close();
                    }

                    if ui.button("Выход").clicked() {
                        ctx.send_viewport_cmd(
                            egui::ViewportCommand::Close,
                        );
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Левая часть
                ui.vertical(|ui| {
                    ui.label("Ключ:");
                    ui.text_edit_singleline(&mut self.key_input);

                    ui.add_space(10.0);

                    ui.label("Исходный текст:");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.input_text)
                            .desired_rows(6)
                    );
                    ui.add_space(10.0);

                    ui.label("Результат:");
                    ui.add_enabled(
                        false,
                        egui::TextEdit::multiline(
                            &mut self.output_text,
                        )
                            .desired_rows(6),
                    );
                });

                ui.add_space(40.0);

                ui.vertical(|ui| {
                    ui.heading("Алгоритм:");

                    if ui
                        .radio_value(
                            &mut self.selected_algorithm,
                            EncryptionAlgorithm::DecimationEnglish,
                            "Метод децимации (английский)",
                        )
                        .clicked()
                    {
                        self.clear_fields();
                    }

                    if ui
                        .radio_value(
                            &mut self.selected_algorithm,
                            EncryptionAlgorithm::VigenereRussian,
                            "Виженер (русский, прямой ключ)",
                        )
                        .clicked()
                    {
                        self.clear_fields();
                    }
                });
            });

            ui.add_space(20.0);

            ui.with_layout(
                Layout::left_to_right(Align::Center),
                |ui| {
                    if ui.button("Зашифровать").clicked() {
                        self.encrypt_action();
                    }

                    if ui.button("Расшифровать").clicked() {
                        self.decrypt_action();
                    }
                },
            );
        });

        if self.error_message.is_some() {
            let error_text = self.error_message.clone().unwrap();
            let mut clear_error = false;

            egui::Window::new("Ошибка")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(error_text);
                    if ui.button("OK").clicked() {
                        clear_error = true;
                    }
                });

            if clear_error {
                self.error_message = None;
            }
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 500.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Лабораторная работа №1 — Шифрование",
        options,
        Box::new(|_cc| Ok(Box::new(CipherApp::default()))),
    )
}