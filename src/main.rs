#![feature(exact_size_is_empty)]
#![feature(new_range_api)]
#![feature(seek_stream_len)]
#![feature(slice_as_array)]

use internal::Directory;
use internal::library::Library;
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

    queue: Vec<u64>,

    play_status: PlayStatus,
    repeat: RepeatStatus,

    mute: bool,
    volume: f32,
}

impl App {
    fn new(stream_handle: rodio::OutputStreamHandle) -> Self {
        let library = internal::scan(
            "/mnt/741ae10f-7ba3-487d-bc13-3953cbb02819/music".into(),
        );

        let sink = rodio::Sink::try_new(&stream_handle).unwrap();

        Self {
            codec_registry: symphonia::default::get_codecs(),
            probe: symphonia::default::get_probe(),
            sink,
            library,
            queue: vec![],
            play_status: PlayStatus::Stopped,
            repeat: RepeatStatus::None,
            mute: false,
            volume: 1.0,
        }
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
                self.sink.stop();
                for track in &self.queue {
                    let track = self.library.get_track(*track).unwrap();
                    self.sink.append(
                        internal::audio::AudioStream::new(
                            &track.path,
                            self.codec_registry,
                            self.probe,
                            track.metadata.duration.unwrap()
                        ));
                }
                self.sink.play();
                self.play_status = PlayStatus::Play;
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
                self.sink.stop();
                for track in &self.queue {
                    let track = self.library.get_track(*track).unwrap();
                    self.sink.append(
                        internal::audio::output::AudioStream::new(
                            &track.path,
                            self.codec_registry,
                            self.probe,
                            track.metadata.duration.unwrap()
                        ));
                }
                self.sink.play();
                self.play_status = PlayStatus::Play;
            }
            Message::Stop => {
                self.play_status = PlayStatus::Stopped;
                self.queue.clear();
                self.sink.stop();
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
}

#[tokio::main]
async fn main() -> iced::Result {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    iced::application("Music player", App::update, App::view)
        .font(view::ICON_FONT_BYTES)
        .theme(theme)
        .run_with(|| {
            (App::new(stream_handle), iced::Task::none())
        })
}

fn theme(_state: &App) -> iced::Theme {
    iced::Theme::Ferra
}
