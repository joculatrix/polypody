#![feature(exact_size_is_empty)]
#![feature(new_range_api)]
#![feature(seek_stream_len)]
#![feature(slice_as_array)]

use internal::Track;
use internal::library::{track_hash, Library};
use std::time::Duration;
use symphonia::core::codecs::CodecRegistry;
use symphonia::core::probe::Probe;

mod internal;
mod view;

#[derive(Debug, Clone, Copy)]
enum Message {
    Dummy,
    PlayFolder,
    PlayheadMoved(f32),
    Shuffle,
    ShuffleFolder,
    SkipBack,
    SkipForward,
    Stop,
    ToggleMute,
    TogglePlay,
    ToggleRepeat,
    UpdateProgress,
    ViewLibrary(u64),
    VolumeChanged(f32),
}

#[derive(Default)]
enum PlayStatus {
    #[default]
    Pause,
    Play,
    Stopped,
}

#[derive(Copy, Clone, Default, PartialEq)]
enum RepeatStatus {
    #[default]
    None,
    One,
    All,
}

struct App {
    codec_registry: &'static CodecRegistry,
    probe: &'static Probe,

    sink: rodio::Sink,
    library: Library,

    playing: Option<Track>,
    queue: Vec<u64>,

    play_status: PlayStatus,
    repeat: RepeatStatus,
    /// If a track is playing, this stores the current timestamp as well as the
    /// total duration of the track.
    track_duration: Option<(Duration, Duration)>,

    mute: bool,
    volume: f32,
}

impl App {
    fn new(stream_handle: rodio::OutputStreamHandle) -> Self {
        let library = internal::scan(
            "/mnt/741ae10f-7ba3-487d-bc13-3953cbb02819/music".into(),
        );

        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        sink.set_volume(0.5);

        Self {
            codec_registry: symphonia::default::get_codecs(),
            probe: symphonia::default::get_probe(),
            sink,
            library,
            playing: None,
            queue: vec![],
            play_status: PlayStatus::Stopped,
            repeat: RepeatStatus::None,
            track_duration: None,
            mute: false,
            volume: 0.5,
        }
    }

    fn play_next(&mut self) {
        let track = self.queue.remove(0);
        let track = self.library.get_track(track).unwrap();
        self.playing = Some(track.clone());
        self.track_duration = track.metadata
            .duration
            .as_ref()
            .map(|total| (Duration::from_secs(0), *total));
        self.sink.append(
            internal::audio::AudioStream::new(
                &track.path,
                self.codec_registry,
                self.probe,
                track.metadata.duration.unwrap(),
            )
        );
        self.sink.play();
        self.play_status = PlayStatus::Play;
    }

    fn stop(&mut self) {
        self.sink.stop();
        self.playing = None;
        self.track_duration = None;
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::PlayFolder => {
                let tracks = &self.library.current_directory().tracks;
                self.queue.resize(tracks.len(), 0);
                self.queue
                    .copy_from_slice(
                        &self.library.current_directory().tracks
                    );
                self.stop();
                self.play_next();
            }
            Message::ShuffleFolder => {
                use rand::{ rng, seq::SliceRandom };

                let tracks = &self.library.current_directory().tracks;
                self.queue.resize(tracks.len(), 0);
                let mut shuffle = (0..tracks.len())
                    .collect::<Vec<usize>>();
                shuffle.shuffle(&mut rng());
                for (i, j) in shuffle.into_iter().enumerate() {
                    self.queue[i] = tracks[j];
                }
                self.stop();
                self.play_next();
            }
            Message::Stop => {
                self.play_status = PlayStatus::Stopped;
                self.queue.clear();
                self.stop();
            }
            Message::ToggleMute => {
                self.mute = !self.mute;
                if self.mute {
                    self.sink.set_volume(0.0);
                } else {
                    self.sink.set_volume(self.volume);
                }
            }
            Message::TogglePlay => {
                self.play_status = match self.play_status {
                    PlayStatus::Play => {
                        self.sink.pause();
                        PlayStatus::Pause
                    }
                    PlayStatus::Pause | PlayStatus::Stopped => {
                        self.sink.play();
                        PlayStatus::Play
                    }
                };
            },
            Message::ToggleRepeat => {
                self.repeat = match self.repeat {
                    RepeatStatus::None => RepeatStatus::One,
                    RepeatStatus::One => RepeatStatus::All,
                    RepeatStatus::All => RepeatStatus::None,
                };
            },
            Message::UpdateProgress => {
                self.update_progress();
            }
            Message::ViewLibrary(id) => {
                self.library.set_current(id); 
            }
            Message::VolumeChanged(val) => {
                self.volume = val;
                self.sink.set_volume(val);
            }
            _ => (),
        }
    }

    fn update_progress(&mut self) {
        if self.sink.empty() {
            if let Some(playing) = &self.playing {
                let last = track_hash(playing);
                match self.repeat {
                    RepeatStatus::One => self.queue.insert(0, last),
                    RepeatStatus::All => self.queue.push(last),
                    RepeatStatus::None => (),
                }
            }
            self.playing = None;
            if self.queue.len() != 0 {
                self.play_next();
            }
        } else {
            let Some(playing) = &self.playing else {
                return; // prevent race conditions
            };
            self.track_duration =
                Some((self.sink.get_pos(), playing.metadata.duration.unwrap()));
        }
    }

    fn progress_subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(Duration::from_millis(10))
            .map(|_| Message::UpdateProgress)
    }
}

#[tokio::main]
async fn main() -> iced::Result {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    iced::application("Music player", App::update, App::view)
        .font(view::ICON_FONT_BYTES)
        .theme(theme)
        .subscription(App::progress_subscription)
        .run_with(|| {
            (App::new(stream_handle), iced::Task::none())
        })
}

fn theme(_state: &App) -> iced::Theme {
    iced::Theme::Ferra
}
