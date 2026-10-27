use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use ankika_core::review::SoundRef;
use rodio::{Decoder, DeviceSinkBuilder, Player};
use tokio::sync::mpsc::UnboundedSender;

use crate::tui::tasks::TaskResult;

/// How often the player checks which sound is playing.
const POLL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioEvent {
    /// The sound now playing, or `None` when the queue ran out or was stopped.
    Playing(Option<SoundRef>),
    /// The file is missing or in a format the decoders don't handle.
    CannotPlay {
        file: String,
    },
    NoOutputDevice,
}

pub struct Track {
    pub sound: SoundRef,
    pub file: String,
}

enum Command {
    Play(Vec<Track>),
    Stop,
}

/// Plays card sounds on its own thread; the audio device handle can't move between threads.
pub struct Audio {
    commands: Sender<Command>,
}

impl Audio {
    pub fn start(media_dir: PathBuf, events: UnboundedSender<TaskResult>) -> Self {
        let (commands, receiver) = mpsc::channel();

        thread::spawn(move || run(&receiver, &media_dir, &events));

        Self { commands }
    }

    /// Replaces whatever is playing, and plays the tracks one after another.
    pub fn play(&self, tracks: Vec<Track>) {
        self.send(Command::Play(tracks));
    }

    pub fn stop(&self) {
        self.send(Command::Stop);
    }

    fn send(&self, command: Command) {
        if self.commands.send(command).is_err() {
            tracing::warn!("The audio thread has stopped");
        }
    }
}

fn run(commands: &Receiver<Command>, media_dir: &Path, events: &UnboundedSender<TaskResult>) {
    let send = |event| {
        // Only fails while the app shuts down
        let _ = events.send(TaskResult::Audio(event));
    };
    // Opened lazily, so cards without sound never touch the audio device
    let mut output = None;
    let mut queue: Vec<SoundRef> = Vec::new();
    let mut playing = None;

    loop {
        match commands.recv_timeout(POLL) {
            Ok(Command::Play(tracks)) => {
                if output.is_none() {
                    output = open_output();
                }

                let Some((_, player)) = &output else {
                    send(AudioEvent::NoOutputDevice);
                    continue;
                };

                player.clear();
                queue.clear();

                for track in tracks {
                    // Anki media names are plain file names; anything else would leave the folder
                    if Path::new(&track.file).file_name() != Some(OsStr::new(&track.file)) {
                        send(AudioEvent::CannotPlay { file: track.file });
                        continue;
                    }

                    match decode(&media_dir.join(&track.file)) {
                        Some(decoder) => {
                            player.append(decoder);
                            queue.push(track.sound);
                        },
                        None => send(AudioEvent::CannotPlay { file: track.file }),
                    }
                }

                player.play();
            },
            Ok(Command::Stop) => {
                if let Some((_, player)) = &output {
                    player.clear();
                }

                queue.clear();
            },
            Err(RecvTimeoutError::Timeout) => {},
            Err(RecvTimeoutError::Disconnected) => return,
        }

        // The player drops sounds from the front as they finish
        let current = output.as_ref().and_then(|(_, player)| {
            queue
                .len()
                .checked_sub(player.len())
                .and_then(|done| queue.get(done))
                .copied()
        });

        if current.is_none() {
            queue.clear();
        }

        if current != playing {
            playing = current;
            send(AudioEvent::Playing(current));
        }
    }
}

fn decode(path: &Path) -> Option<Decoder<BufReader<File>>> {
    let file = File::open(path)
        .inspect_err(|error| tracing::warn!(?path, ?error, "Cannot open a sound"))
        .ok()?;

    Decoder::try_from(file)
        .inspect_err(|error| tracing::warn!(?path, ?error, "Cannot decode a sound"))
        .ok()
}

fn open_output() -> Option<(rodio::MixerDeviceSink, Player)> {
    match DeviceSinkBuilder::open_default_sink() {
        Ok(mut sink) => {
            sink.log_on_drop(false);
            let player = Player::connect_new(sink.mixer());

            Some((sink, player))
        },
        Err(error) => {
            tracing::warn!(?error, "No audio output device");
            None
        },
    }
}
