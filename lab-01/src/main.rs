#![windows_subsystem = "windows"]
use eframe::egui;
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

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {

                    ui.label("Ключ:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.key_input)
                            .desired_width(f32::INFINITY)
                    );

                    ui.add_space(10.0);

                    // ===== Исходный текст =====
                    ui.collapsing("Исходный текст", |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.input_text)
                                .desired_width(f32::INFINITY)
                                .desired_rows(15)
                        );
                    });

                    ui.add_space(10.0);

                    ui.collapsing("Результат", |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.output_text)
                                .desired_width(f32::INFINITY)
                                .desired_rows(15)
                                .interactive(false)
                        );
                    });

                    ui.add_space(15.0);

                    ui.separator();

                    ui.heading("Алгоритм");

                    if ui.radio_value(
                        &mut self.selected_algorithm,
                        EncryptionAlgorithm::DecimationEnglish,
                        "Метод децимации (английский)",
                    ).clicked() {
                        self.output_text.clear();
                    }

                    if ui.radio_value(
                        &mut self.selected_algorithm,
                        EncryptionAlgorithm::VigenereRussian,
                        "Виженер (русский)",
                    ).clicked() {
                        self.output_text.clear();
                    }

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if ui.button("Зашифровать").clicked() {
                            self.encrypt_action();
                        }

                        if ui.button("Расшифровать").clicked() {
                            self.decrypt_action();
                        }
                    });
                });
        });

        if let Some(error_text) = self.error_message.clone() {
            let mut clear_error = false;

            egui::Window::new("Ошибка")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .fixed_size([420.0, 180.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);

                        ui.heading("⚠ Ошибка");

                        ui.add_space(10.0);

                        ui.label(
                            egui::RichText::new(error_text)
                                .size(18.0)
                        );

                        ui.add_space(20.0);

                        if ui
                            .add_sized(
                                [120.0, 35.0],
                                egui::Button::new("OK"),
                            )
                            .clicked()
                        {
                            clear_error = true;
                        }
                    });
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