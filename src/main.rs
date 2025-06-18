#![feature(exact_size_is_empty)]
#![feature(new_range_api)]
#![feature(seek_stream_len)]
#![feature(slice_as_array)]

use internal::Directory;
use internal::library::Library;

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
    library: Library,

    queue: Vec<u64>,

    play_status: PlayStatus,
    repeat: RepeatStatus,

    mute: bool,
    volume: f32,
}

impl Default for App {
    fn default() -> Self {
        let library = internal::scan(
            "/mnt/741ae10f-7ba3-487d-bc13-3953cbb02819/music".into(),
        );

        Self {
            library,
            queue: vec![],
            play_status: PlayStatus::Stopped,
            repeat: RepeatStatus::None,
            mute: false,
            volume: 1.0,
        }
    }
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::PlayFolder => {
                let tracks = &self.library.current_directory().tracks;
                self.queue.resize(tracks.len(), 0);
                self.queue
                    .copy_from_slice(
                        &self.library.current_directory().tracks
                    );
                // TODO: start playing first track in new queue
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
                // TODO: start playing first track in new queue
            }
            Message::Stop => {
                self.play_status = PlayStatus::Stopped;
                self.queue.clear();
                // TODO: stop playback
            }
            Message::ToggleMute => {
                self.mute = !self.mute;
            }
            Message::TogglePlay => {
                self.play_status = match self.play_status {
                    PlayStatus::Play => PlayStatus::Pause,
                    PlayStatus::Pause | PlayStatus::Stopped => PlayStatus::Play,
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
            }
            _ => (),
        }
    }
}

fn main() -> iced::Result {
    iced::application("Music player", App::update, App::view)
        .font(view::ICON_FONT_BYTES)
        .theme(theme)
        .run()
}

fn theme(_state: &App) -> iced::Theme {
    iced::Theme::Ferra
}
