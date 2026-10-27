use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use toml_edit::{DocumentMut, value};

use crate::cli::ImageMode;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to parse {}", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("Failed to edit {}", path.display())]
    Edit {
        path: PathBuf,
        #[source]
        source: toml_edit::TomlError,
    },
    #[error("Failed to write {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeName {
    #[default]
    Dark,
    Light,
}

impl ThemeName {
    const fn key(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
}

/// ankika's settings file. Command line flags and environment variables override it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub theme: ThemeName,
    pub sync_on_start: bool,
    pub sync_on_quit: bool,
    pub images: ImageMode,
    pub sync_server: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeName::default(),
            sync_on_start: true,
            sync_on_quit: true,
            images: ImageMode::Auto,
            sync_server: None,
        }
    }
}

/// Every key is optional, so a partial file keeps the other defaults.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    theme: Option<ThemeName>,
    sync_on_start: Option<bool>,
    sync_on_quit: Option<bool>,
    images: Option<ImageMode>,
    sync_server: Option<String>,
}

/// The defaults when the file doesn't exist yet.
pub fn load(path: &Path) -> Result<Config, ConfigError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(source) => {
            return Err(ConfigError::Read {
                path: path.to_owned(),
                source,
            });
        },
    };
    let file: File = toml::from_str(&text).map_err(|source| ConfigError::Parse {
        path: path.to_owned(),
        source,
    })?;
    let defaults = Config::default();

    Ok(Config {
        theme: file.theme.unwrap_or(defaults.theme),
        sync_on_start: file.sync_on_start.unwrap_or(defaults.sync_on_start),
        sync_on_quit: file.sync_on_quit.unwrap_or(defaults.sync_on_quit),
        images: file.images.unwrap_or(defaults.images),
        sync_server: file.sync_server.filter(|server| !server.trim().is_empty()),
    })
}

/// Writes the values into the file, keeping its comments and layout.
pub fn save(path: &Path, config: &Config) -> Result<(), ConfigError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(source) => {
            return Err(ConfigError::Read {
                path: path.to_owned(),
                source,
            });
        },
    };
    let mut document: DocumentMut = text.parse().map_err(|source| ConfigError::Edit {
        path: path.to_owned(),
        source,
    })?;

    document["theme"] = value(config.theme.key());
    document["sync_on_start"] = value(config.sync_on_start);
    document["sync_on_quit"] = value(config.sync_on_quit);
    document["images"] = value(image_mode_key(config.images));

    match &config.sync_server {
        Some(server) => document["sync_server"] = value(server.as_str()),
        None => {
            document.remove("sync_server");
        },
    }

    let write_error = |source| ConfigError::Write {
        path: path.to_owned(),
        source,
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(write_error)?;
    }

    fs::write(path, document.to_string()).map_err(write_error)
}

/// The spelling the file and `--images` use.
pub const fn image_mode_key(mode: ImageMode) -> &'static str {
    match mode {
        ImageMode::Auto => "auto",
        ImageMode::Kitty => "kitty",
        ImageMode::Sixel => "sixel",
        ImageMode::Iterm2 => "iterm2",
        ImageMode::Halfblocks => "halfblocks",
        ImageMode::Off => "off",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ankika-config-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("config.toml")
    }

    #[test]
    fn missing_file_gives_defaults() {
        assert_eq!(load(&temp_file("missing")).unwrap(), Config::default());
    }

    #[test]
    fn saving_keeps_comments_and_reads_back() {
        let path = temp_file("save");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "# my settings\ntheme = \"dark\"\n").unwrap();

        let config = Config {
            theme: ThemeName::Light,
            sync_on_quit: false,
            images: ImageMode::Halfblocks,
            sync_server: Some("http://localhost:8080/".to_owned()),
            ..Config::default()
        };
        save(&path, &config).unwrap();

        assert!(
            fs::read_to_string(&path)
                .unwrap()
                .starts_with("# my settings\n")
        );
        assert_eq!(load(&path).unwrap(), config);

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn partial_file_keeps_other_defaults() {
        let path = temp_file("partial");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "sync_on_start = false\n").unwrap();

        let config = load(&path).unwrap();
        assert!(!config.sync_on_start);
        assert!(config.sync_on_quit);

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
