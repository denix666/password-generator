use std::{io::Write, path::PathBuf};
use serde::{Deserialize, Serialize};
use toml::to_string;

#[derive(Deserialize, Serialize, Clone)]
pub struct Options {
    pub last_window_pos_x: f32,
    pub last_window_pos_y: f32,
    pub password_length: usize,
    pub use_letters: bool,
    pub use_numbers: bool,
    pub use_special_chars: bool,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub options: Options,
}


pub fn app_root_path() -> PathBuf {
    let mut app_root_path = match home::home_dir() {
        Some(path) => path,
        None => panic!("Impossible to get your home dir!"),
    };
    app_root_path.push(crate::CONFIG_DIR);
    return app_root_path
}

pub fn write_config_to_file(
    last_window_pos_x: f32,
    last_window_pos_y: f32,
    password_length: usize,
    use_letters: bool,
    use_numbers: bool,
    use_special_chars: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_file_path = app_root_path();
    config_file_path.push(crate::MAIN_CONFIG_FILE_NAME);

    if let Some(parent) = config_file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let app_config = Config {
        options: Options {
            last_window_pos_x,
            last_window_pos_y,
            password_length,
            use_letters,
            use_numbers,
            use_special_chars,
        }
    };

    let toml_string = toml::to_string(&app_config)?;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&config_file_path)?;

    file.write_all(toml_string.as_bytes())?;
    file.flush()?;

    Ok(())
}

pub fn read_app_config_from_file() -> Config {
    let mut config_file_path = app_root_path();
    config_file_path.push(crate::MAIN_CONFIG_FILE_NAME);

    let new_config = Config {
        // Default configuation
        options: Options {
            last_window_pos_x: 20.0,
            last_window_pos_y: 10.0,
            password_length: 10,
            use_letters: true,
            use_numbers: true,
            use_special_chars: true,
        },
    };

    let toml_str = match std::fs::read_to_string(config_file_path) {
        Ok(res) => res,
        Err(_) => {
            write_config_to_file(
                20.0,
                10.0,
                10,
                true,
                true,
                true,
            ).unwrap();
            to_string(&new_config).unwrap()
        }
    };

    // Try to use loaded config. In case of error - create new (backward compatibility)
    let app_config = match toml::from_str(&toml_str) {
        Ok(res) => res,
        Err(_) => {
            write_config_to_file(
                20.0,
                10.0,
                10,
                true,
                true,
                true,
            ).unwrap();
            new_config
        }
    };

    app_config
}


pub fn window_moved(ctx: &eframe::egui::Context, app_config: &mut Config) -> bool {
    let mut changed = false;

    ctx.input(|i| {
        if let Some(rect) = i.viewport().outer_rect {
            let pos_x = rect.min.x;
            let pos_y = rect.min.y;

            if app_config.options.last_window_pos_x != pos_x {
                app_config.options.last_window_pos_x = pos_x;
                changed = true;
            }

            if app_config.options.last_window_pos_y != pos_y {
                app_config.options.last_window_pos_y = pos_y;
                changed = true;
            }
        }
    });

    return changed
}
