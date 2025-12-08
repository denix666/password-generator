use eframe::*;
use egui::{Color32, RichText, Slider};
use rand::Rng;
use arboard::Clipboard;

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
}

impl PasswordApp {
    fn new() -> Self {
        Self {
            password: String::new(),
            password_length: 10, // Default length
            use_letters: true,  // Default: enabled
            use_numbers: true,  // Default: enabled
            use_special_chars: true, // Default: enabled
            copy_status: String::new(),
            copy_status_time: None,
        }
    }

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
            available_chars.extend("!@#*()_[]{},".chars());
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
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([430.0, 235.0]).with_maximize_button(false),
        ..Default::default()
    };

    if let Ok(icon) = load_embedded_icon() {
        options.viewport = options.viewport.with_icon(icon);
    }


    let mut password_app = PasswordApp::new();

    eframe::run_simple_native(&title, options, move |ctx, _frame| {
        ctx.set_visuals(egui::Visuals::dark());
        egui::CentralPanel::default().show(&ctx, |_ui| {

            // Проверяем, нужно ли скрыть сообщение о копировании
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
                                //ui.label(format!("{}", password_app.password_length));
                            });

                            // Character types
                            ui.checkbox(&mut password_app.use_letters, "Include letters (a-z, A-Z)");
                            ui.checkbox(&mut password_app.use_numbers, "Include numbers (0-9)");
                            ui.checkbox(&mut password_app.use_special_chars, "Include special characters ( !@#*()_[]{}, )");
                        });
                    });

                    ui.add_space(20.0);

                    if ui.button(RichText::new("Generate Password").size(20.0)).clicked() {
                        password_app.generate_password();
                    }
                });
            });

            if password_app.copy_status_time.is_some() {
                ctx.request_repaint();
            }
        });
    }).unwrap();
}
