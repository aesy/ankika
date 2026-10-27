use crate::cli::ImageMode;
use crate::config::{self, Config, ThemeName};
use crate::i18n::I18n;
use crate::tui::form::{Field, Form, Value};

const THEMES: [ThemeName; 2] = [ThemeName::Dark, ThemeName::Light];
const IMAGE_MODES: [ImageMode; 6] = [
    ImageMode::Auto,
    ImageMode::Kitty,
    ImageMode::Sixel,
    ImageMode::Iterm2,
    ImageMode::Halfblocks,
    ImageMode::Off,
];

// The fields' positions in the form
const THEME: usize = 0;
const SYNC_ON_START: usize = 1;
const SYNC_ON_QUIT: usize = 2;
const IMAGES: usize = 3;
const SYNC_SERVER: usize = 4;

/// ankika's settings file as a form.
pub fn new(config: &Config, i18n: &I18n) -> Form {
    let theme_labels = THEMES
        .iter()
        .map(|theme| match theme {
            ThemeName::Dark => i18n.text("settings-theme-dark"),
            ThemeName::Light => i18n.text("settings-theme-light"),
        })
        .collect();
    let fields = vec![
        Field {
            label: i18n.text("settings-theme"),
            value: Value::Choice {
                options: theme_labels,
                selected: THEMES
                    .iter()
                    .position(|theme| *theme == config.theme)
                    .unwrap_or(0),
            },
        },
        Field {
            label: i18n.text("settings-sync-on-start"),
            value: Value::Toggle(config.sync_on_start),
        },
        Field {
            label: i18n.text("settings-sync-on-quit"),
            value: Value::Toggle(config.sync_on_quit),
        },
        Field {
            label: i18n.text("settings-images"),
            value: Value::Choice {
                options: IMAGE_MODES
                    .iter()
                    .map(|&mode| config::image_mode_key(mode).to_owned())
                    .collect(),
                selected: IMAGE_MODES
                    .iter()
                    .position(|mode| *mode == config.images)
                    .unwrap_or(0),
            },
        },
        Field {
            label: i18n.text("settings-sync-server"),
            value: Value::Text(config.sync_server.clone().unwrap_or_default()),
        },
    ];
    let mut form = Form::new(i18n.text("settings-title"), fields);

    form.note = Some(i18n.text("settings-restart-note"));
    form
}

pub fn read(form: &Form) -> Config {
    let server = form.text(SYNC_SERVER).trim();

    Config {
        theme: THEMES[form.choice(THEME).min(THEMES.len() - 1)],
        sync_on_start: form.toggle(SYNC_ON_START),
        sync_on_quit: form.toggle(SYNC_ON_QUIT),
        images: IMAGE_MODES[form.choice(IMAGES).min(IMAGE_MODES.len() - 1)],
        sync_server: (!server.is_empty()).then(|| server.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_the_config() {
        let config = Config {
            theme: ThemeName::Light,
            sync_on_start: false,
            sync_on_quit: true,
            images: ImageMode::Off,
            sync_server: Some("http://localhost/".to_owned()),
        };

        assert_eq!(read(&new(&config, &I18n::new().unwrap())), config);
    }
}
