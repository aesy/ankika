use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use anki::card::CardId;
use ratatui::Frame;
use ratatui::layout::{Rect, Size};
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{Resize, StatefulImage};

use crate::cli::ImageMode;

const RESIZE: Resize = Resize::Fit(None);

/// Decoded images of the card on screen, drawn with the terminal's graphics protocol.
pub struct Images {
    picker: Option<Picker>,
    media_dir: PathBuf,
    /// Rendering needs mutable protocol state, but views only get shared references.
    cache: Mutex<Cache>,
}

#[derive(Default)]
struct Cache {
    card: Option<CardId>,
    /// `None` when the file could not be loaded; the view then shows a placeholder.
    images: HashMap<String, Option<StatefulProtocol>>,
}

impl Images {
    /// Asks the terminal what it supports, so it must run after entering the alternate
    /// screen and before reading events.
    pub fn detect(mode: ImageMode, media_dir: PathBuf) -> Self {
        let forced = match mode {
            ImageMode::Off => None,
            ImageMode::Halfblocks => Some(Picker::halfblocks()),
            ImageMode::Auto => Some(query()),
            ImageMode::Kitty => Some(forced(ProtocolType::Kitty)),
            ImageMode::Sixel => Some(forced(ProtocolType::Sixel)),
            ImageMode::Iterm2 => Some(forced(ProtocolType::Iterm2)),
        };

        if let Some(picker) = &forced {
            tracing::info!(protocol = ?picker.protocol_type(), "Image protocol");
        }

        Self {
            picker: forced,
            media_dir,
            cache: Mutex::default(),
        }
    }

    /// Drops the previous card's images when another card is shown.
    pub fn show_card(&self, card: CardId) {
        let mut cache = self.cache();

        if cache.card != Some(card) {
            *cache = Cache {
                card: Some(card),
                images: HashMap::new(),
            };
        }
    }

    /// The cells the image takes when fitted into `available`, or `None` when it can't
    /// be drawn.
    pub fn size(&self, src: &str, available: Size) -> Option<Size> {
        let mut cache = self.cache();

        self.load(&mut cache, src)
            .map(|protocol| protocol.size_for(RESIZE, available))
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, src: &str) {
        let mut cache = self.cache();

        if let Some(protocol) = self.load(&mut cache, src) {
            frame.render_stateful_widget(StatefulImage::new().resize(RESIZE), area, protocol);

            if let Some(Err(error)) = protocol.last_encoding_result() {
                tracing::warn!(src, ?error, "Failed to draw an image");
            }
        }
    }

    /// A panic while drawing leaves nothing half-updated that matters, so poison is ignored.
    fn cache(&self) -> MutexGuard<'_, Cache> {
        self.cache.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn load<'a>(&self, cache: &'a mut Cache, src: &str) -> Option<&'a mut StatefulProtocol> {
        let picker = self.picker.as_ref()?;

        // Anki media names are plain file names; anything else would leave the media folder
        if Path::new(src).file_name() != Some(OsStr::new(src)) {
            return None;
        }

        cache
            .images
            .entry(src.to_owned())
            .or_insert_with(|| {
                let path = self.media_dir.join(src);

                image::ImageReader::open(&path)
                    .and_then(image::ImageReader::with_guessed_format)
                    .map_err(image::ImageError::IoError)
                    .and_then(image::ImageReader::decode)
                    .inspect_err(|error| tracing::warn!(src, ?error, "Failed to load an image"))
                    .ok()
                    .map(|image| picker.new_resize_protocol(image))
            })
            .as_mut()
    }
}

/// Falls back to half blocks when the terminal can't be asked.
fn query() -> Picker {
    Picker::from_query_stdio().unwrap_or_else(|error| {
        tracing::warn!(?error, "Failed to query the terminal's image support");
        Picker::halfblocks()
    })
}

fn forced(protocol: ProtocolType) -> Picker {
    let mut picker = query();
    picker.set_protocol_type(protocol);
    picker
}
