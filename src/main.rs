#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::*;
use egui::{Color32, RichText, Slider};
use rand::Rng;
use arboard::Clipboard;

mod config;
use config::*;

const CONFIG_DIR: &str = ".local/share/password-generator";
const MAIN_CONFIG_FILE_NAME: &str = "config.toml";
const ICON_BYTES: &[u8] = include_bytes!("../pkg/password-generator.png");

pub fn load_embedded_icon() -> Result<crate::egui::IconData, String> {
    let img = image::load_from_memory(ICON_BYTES).map_err(|e| e.to_string())?.into_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();
    Ok(crate::egui::IconData { rgba, width, height })
}

struct PasswordApp {
    password: String,
    password_length: usize,
    use_letters: bool,
    use_numbers: bool,
    use_special_chars: bool,
    copy_status: String,
    copy_status_time: Option<std::time::Instant>,
    special_chars_set: String,
}

impl PasswordApp {
    fn generate_password(&mut self) {
        let mut rng = rand::rng();
        let mut available_chars = Vec::new();

        if self.use_letters {
            available_chars.extend('a'..='z');
            available_chars.extend('A'..='Z');
        }

        if self.use_numbers {
            available_chars.extend('0'..='9');
        }

        if self.use_special_chars {
            available_chars.extend(self.special_chars_set.chars());
        }

        // Если ни один из наборов символов не выбран, используем только буквы
        if available_chars.is_empty() {
            available_chars.extend('a'..='z');
            available_chars.extend('A'..='Z');
            self.use_letters = true;
        }

        // Генерируем пароль
        self.password = (0..self.password_length)
            .map(|_| {
                let idx = Rng::random_range(&mut rng, 0..available_chars.len());
                available_chars[idx]
            })
            .collect();
    }

    fn copy_to_clipboard(&mut self) -> Result<(), String> {
        if self.password.is_empty() {
            return Err("No password to copy".to_string());
        }

        match Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(&self.password) {
                    return Err(format!("Failed to copy: {}", e));
                }
                self.copy_status = "Password copied!".to_string();
                self.copy_status_time = Some(std::time::Instant::now());
                Ok(())
            }
            Err(e) => Err(format!("Clipboard error: {}", e)),
        }
    }
}

fn main() {
    let mut title = String::from("Password generator v");
    title.push_str(env!("CARGO_PKG_VERSION"));

    // App config
    let mut app_config = read_app_config_from_file();
    let mut config_should_be_saved = false;

    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([430.0, 235.0]).with_maximize_button(false)
            .with_position(egui::Pos2::new(app_config.options.last_window_pos_x, app_config.options.last_window_pos_y)),
        ..Default::default()
    };

    if let Ok(icon) = load_embedded_icon() {
        options.viewport = options.viewport.with_icon(icon);
    }

    let mut password_app = PasswordApp {
        password: String::new(),
        password_length: app_config.options.password_length,
        use_letters: app_config.options.use_letters,
        use_numbers: app_config.options.use_numbers,
        use_special_chars: app_config.options.use_special_chars,
        copy_status: String::new(),
        copy_status_time: None,
        special_chars_set: "!@#*()_[]{},".to_string(),
    };

    eframe::run_simple_native(&title, options, move |ctx, _frame| {
        ctx.set_visuals(egui::Visuals::dark());

        // Manage window position and size
        if window_moved(ctx, &mut app_config) {
            config_should_be_saved = true;
        }

        egui::CentralPanel::default().show(&ctx, |_ui| {

            // Check if we should hide the copy message
            if let Some(time) = password_app.copy_status_time {
                if time.elapsed().as_secs() >= 3 {
                    password_app.copy_status.clear();
                    password_app.copy_status_time = None;
                }
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Password: ");
                        let text_edit = egui::TextEdit::singleline(&mut password_app.password)
                            .desired_width(300.0);
                        ui.add(text_edit);

                        if ui.button("📋").on_hover_text("Copy to clipboard").clicked() {
                            if let Err(e) = password_app.copy_to_clipboard() {
                                password_app.copy_status = e;
                                password_app.copy_status_time = Some(std::time::Instant::now());
                            }
                        }
                    });

                    if !password_app.copy_status.is_empty() {
                        ui.colored_label(Color32::GREEN, &password_app.copy_status);
                    }

                    ui.add_space(20.0);

                    ui.group(|ui| {
                        ui.label("Password Settings");

                        // Length slider
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Length: ");
                                ui.add(Slider::new(&mut password_app.password_length, 4..=32));
                            });

                            // Character types
                            ui.checkbox(&mut password_app.use_letters, "Include letters (a-z, A-Z)");
                            ui.checkbox(&mut password_app.use_numbers, "Include numbers (0-9)");
                            ui.checkbox(&mut password_app.use_special_chars, format!("Include special characters ( {} )", password_app.special_chars_set));
                        });
                    });

                    ui.add_space(20.0);

                    if ui.button(RichText::new("Generate Password").size(20.0)).clicked() {
                        password_app.generate_password();
                    }
                });
            });

            if password_app.password_length != app_config.options.password_length {
                config_should_be_saved = true;
                app_config.options.password_length = password_app.password_length;
            }

            if password_app.use_letters != app_config.options.use_letters {
                config_should_be_saved = true;
                app_config.options.use_letters = password_app.use_letters;
            }

            if password_app.use_numbers != app_config.options.use_numbers {
                config_should_be_saved = true;
                app_config.options.use_numbers = password_app.use_numbers;
            }

            if password_app.use_special_chars != app_config.options.use_special_chars {
                config_should_be_saved = true;
                app_config.options.use_special_chars = password_app.use_special_chars;
            }

            if config_should_be_saved {
                let _ = write_config_to_file(
                    app_config.options.last_window_pos_x,
                    app_config.options.last_window_pos_y,
                    app_config.options.password_length,
                    app_config.options.use_letters,
                    app_config.options.use_numbers,
                    app_config.options.use_special_chars,
                );
                config_should_be_saved = false;
            }

            if password_app.copy_status_time.is_some() {
                ctx.request_repaint();
            }
        });
    }).unwrap();
}
