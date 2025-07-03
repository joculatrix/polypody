use super::*;
use config::Config;
use playlist::{Playlist, PlaylistMap, PlaylistTrack};
pub use view::ICON_FONT_BYTES;

use iced::{task::Task, widget::combo_box};

use view::start_screen;

mod config;
mod playlist;
mod view;

#[derive(Debug, Clone)]
pub enum Message {
    AppendTrack(u64),
    CancelCreatePlaylist,
    CloseAddToPlaylist,
    CreatePlaylist,
    ImgPathChanged(String),
    ImgSelected(Option<rfd::FileHandle>),
    None,
    OpenImgDialog,
    OpenNewPlaylist,
    PinAdd(PinKind, PathBuf),
    PinRemove(PinKind, usize),
    PinSwap(PinKind, usize, usize),
    PlayFolder,
    PlayList,
    PlaylistPathChanged(String),
    PlaylistRemove(usize),
    PlaylistSelected(u64),
    PlaylistSwap(usize, usize),
    PlaylistTitleChanged(String),
    PlayTrack(u64),
    PlayheadMoved(f32),
    PlayheadReleased,
    QueueRemove(usize),
    QueueSwap(usize, usize),
    ScanDone,
    SelectPlaylist(u64),
    Shuffle,
    ShuffleFolder,
    ShuffleList,
    SkipBack,
    SkipForward,
    StartScreen(start_screen::Message),
    Stop,
    ToggleMute,
    TogglePlay,
    ToggleRepeat,
    UpdateProgress,
    ViewLibrary(u64),
    ViewPlaylist(Option<u64>),
    VolumeChanged(f32),
}

#[derive(Copy, Clone, Debug)]
pub enum PinKind {
    Library,
    Playlist,
}

#[derive(Copy, Clone, Default)]
pub enum PlayStatus {
    #[default]
    Pause,
    Play,
    Stopped,
}

#[derive(Copy, Clone, Default, PartialEq)]
pub enum RepeatStatus {
    #[default]
    None,
    One,
    All,
}

pub enum Viewing {
    Library,
    Playlist(Option<u64>),
}

pub struct App {
    codec_registry: &'static CodecRegistry,
    probe: &'static Probe,

    config: Config,
    library: Library,
    playlists: PlaylistMap,
    viewing: Viewing,

    sink: rodio::Sink,
    playing: Option<Track>,
    queue: Vec<u64>,
    play_status: PlayStatus,
    repeat: RepeatStatus,
    playhead_position: f32,
    seeking: bool,
    /// If a track is playing, this stores the current timestamp as well as the
    /// total duration of the track.
    track_duration: Option<(Duration, Duration)>,

    mute: bool,
    volume: f32,
    start_screen: Option<start_screen::StartScreen>,

    selecting_playlist: Option<u64>,

    new_playlist_menu: bool,
    new_playlist_title: String,
    new_playlist_path: String,
    new_playlist_img: String,
}

impl App {
    pub fn new(stream_handle: rodio::OutputStreamHandle) -> Self {
        let (config, library, start_screen) =
            match Config::from_file(Config::file_path().unwrap())
        {
            Ok(config) => {
                let lib_cache_path = Library::file_path();
                if let Ok(path) = lib_cache_path && path.exists() {
                    let lib = {
                        let config_lib_path = config.library.path.clone();
                        Library::from_file(&path)
                            .map_or_else(
                                |_| internal::scan(&config_lib_path),
                                |lib|
                                    if lib.root_directory().path ==
                                        config_lib_path
                                        && !config.library.full_rescan_on_start
                                    {
                                        internal::partial_scan(&config_lib_path, lib)
                                    } else {
                                        internal::scan(&config_lib_path)
                                    }
                            )
                    };
                    (config, lib, None)
                } else {
                    (
                        config,
                        Library::new(),
                        Some(start_screen::StartScreen::new())
                    )
                }
            }
            Err(e) => {
                eprintln!("Couldn't read config: {e}");
                (
                    Config::default(),
                    Library::new(),
                    Some(start_screen::StartScreen::new())
                )
            }
        };

        let mut playlists = PlaylistMap::new();
        playlists.scan_playlists();

        let add_to_playlist_state = combo_box::State
            ::new(playlists.playlists().map(|(id, _)| *id).collect());

        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        let volume = config.misc.default_volume.min(1.0).max(0.0);
        sink.set_volume(volume);

        Self {
            codec_registry: symphonia::default::get_codecs(),
            probe: symphonia::default::get_probe(),
            config,
            library,
            playlists,
            viewing: Viewing::Library,
            sink,
            playing: None,
            queue: vec![],
            playhead_position: 0.0,
            seeking: false,
            play_status: PlayStatus::Stopped,
            repeat: RepeatStatus::None,
            track_duration: None,
            mute: false,
            volume,
            start_screen,
            selecting_playlist: None,
            new_playlist_menu: false,
            new_playlist_title: String::new(),
            new_playlist_path: String::new(),
            new_playlist_img: String::new(),
        }
    }

    fn play_next(&mut self) {
        if self.queue.len() == 0 {
            return;
        }
        let track = self.queue.remove(0);
        let track = self.library.get_track(track).unwrap();
        self.playing = Some(track.clone());
        self.playhead_position = 0.0;
        self.track_duration = track.metadata
            .duration
            .as_ref()
            .map(|total| (Duration::from_secs(0), *total));
        self.sink.stop();
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
        self.play_status = PlayStatus::Stopped;
    }

    fn write_config(&self) -> Task<Message> {
        let config = self.config.clone();
        Task::future(tokio::spawn(async move {
            config.write_to_file(&Config::file_path().unwrap());
        }))
            .map(|_| Message::None)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AppendTrack(id) => {
                self.queue.push(id);
                Task::none()
            }
            Message::CancelCreatePlaylist => {
                self.new_playlist_menu = false;
                Task::none()
            }
            Message::CloseAddToPlaylist => {
                self.selecting_playlist = None;
                Task::none()
            }
            Message::CreatePlaylist => {
                let img = {
                    let path = PathBuf::from(&self.new_playlist_img);
                    if path.exists() {
                        Some(path)
                    } else {
                        None
                    }
                };
                let pl = Playlist::new(
                    self.new_playlist_title.clone(),
                    format!("{}.toml", self.new_playlist_path),
                    img,
                    vec![],
                );
                pl.write_to_file();
                self.playlists.scan_playlists();
                self.new_playlist_menu = false;
                Task::none()
            }
            Message::ImgPathChanged(s) => {
                self.new_playlist_img = s;
                Task::none()
            }
            Message::ImgSelected(fh) => {
                if let Some(fh) = fh {
                    self.new_playlist_img = fh.path().to_str().unwrap().to_owned();
                }
                Task::none()
            }
            Message::OpenImgDialog => {
                Task::perform(
                    rfd::AsyncFileDialog::new()
                        .add_filter("image", &["png", "jpg", "jpeg"])
                        .pick_file(),
                    Message::ImgSelected,
                )
            }
            Message::OpenNewPlaylist => {
                self.new_playlist_title.clear();
                self.new_playlist_img.clear();
                self.new_playlist_path.clear();
                self.new_playlist_menu = true;
                Task::none()
            }
            Message::PinAdd(kind, path) => {
                match kind {
                    PinKind::Library => {
                        self.config.library.pins.push(path);
                    }
                    PinKind::Playlist => {
                        self.config.playlists.pins.push(path);
                    }
                }
                self.write_config()
            }
            Message::PinRemove(kind, index) => {
                match kind {
                    PinKind::Library => {
                        self.config.library.pins.remove(index);
                    }
                    PinKind::Playlist => {
                        self.config.playlists.pins.remove(index);
                    }
                }
                self.write_config()
            }
            Message::PinSwap(kind, a, b) => {
                match kind {
                    PinKind::Library => {
                        if b < self.config.library.pins.len() {
                            self.config.library.pins.swap(a, b);
                        }
                    }
                    PinKind::Playlist => {
                        if b < self.config.playlists.pins.len() {
                            self.config.playlists.pins.swap(a, b);
                        }
                    }
                }
                self.write_config()
            }
            Message::PlayFolder => {
                let tracks = &self.library.current_directory().tracks;
                self.queue.resize(tracks.len(), 0);
                self.queue.copy_from_slice(tracks);
                self.stop();
                self.play_next();
                Task::none()
            }
            Message::PlayList => {
                let Viewing::Playlist(Some(list)) = self.viewing else {
                    return Task::none();
                };
                let tracks = self.playlists.get_playlist(list).unwrap()
                    .tracks
                    .iter()
                    .map(|track| match track {
                        PlaylistTrack::Track(id, _) => Some(*id),
                        _ => None,
                    })
                    .flatten()
                    .collect::<Vec<_>>();
                self.queue.resize(tracks.len(), 0);
                self.queue.copy_from_slice(&tracks);
                self.stop();
                self.play_next();
                Task::none()
            }
            Message::PlaylistPathChanged(s) => {
                self.new_playlist_path = s;
                Task::none()
            }
            Message::PlaylistRemove(index) => unsafe {
                let Viewing::Playlist(Some(id)) = self.viewing else {
                    return Task::none();
                };
                let pl = self.playlists.get_playlist_mut(id).unwrap_unchecked();
                pl.tracks.remove(index);
                pl.write_to_file();
                Task::none()
            }
            Message::PlaylistSelected(pl_id) => unsafe {
                let track_id = self.selecting_playlist.unwrap_unchecked();
                let pl = self.playlists.get_playlist_mut(pl_id).unwrap_unchecked();
                pl.tracks.push(PlaylistTrack::Track(
                    track_id,
                    self.library.get_track(track_id).unwrap_unchecked()
                        .path.clone(),
                ));
                self.selecting_playlist = None;
                pl.write_to_file();
                Task::none()
            }
            Message::PlaylistSwap(a, b) => unsafe {
                let Viewing::Playlist(Some(id)) = self.viewing else {
                    return Task::none();
                };
                let pl = self.playlists.get_playlist_mut(id).unwrap_unchecked();
                if b < pl.tracks.len() {
                    pl.tracks.swap(a, b);
                }
                pl.write_to_file();
                Task::none()
            }
            Message::PlaylistTitleChanged(s) => {
                self.new_playlist_title = s;
                Task::none()
            }
            Message::PlayTrack(id) => {
                self.queue.clear();
                self.queue.push(id);
                self.play_next();
                Task::none()
            }
            Message::PlayheadMoved(val) => {
                self.playhead_position = val;
                self.seeking = true;

                let Some(_playing) = &self.playing else {
                    return Task::none();
                };

                let Some((_, duration)) = &self.track_duration else {
                    return Task::none();
                };

                let seek_pos = Duration::from_secs(
                    (val * duration.as_secs_f32()) as u64);
                self.sink.try_seek(seek_pos);
                Task::none()
            }
            Message::PlayheadReleased => {
                self.seeking = false;
                Task::none()
            }
            Message::QueueRemove(index) => {
                self.queue.remove(index);
                Task::none()
            }
            Message::QueueSwap(a, b) => {
                if b < self.queue.len() {
                    self.queue.swap(a, b);
                }
                Task::none()
            }
            Message::ScanDone => unsafe {
                let start = self.start_screen.take().unwrap_unchecked();
                self.library = start.lib.unwrap_unchecked();
                let _ = self.library.write_to_file()
                    .inspect_err(|e| eprintln!("Problem caching library data: {e}"));
                self.config.library.path = start.path.into();
                self.write_config()
            }
            Message::SelectPlaylist(track_id) => {
                self.selecting_playlist = Some(track_id);
                Task::none()
            }
            Message::Shuffle => {
                use rand::{ rng, seq::SliceRandom };

                self.queue.shuffle(&mut rng());
                Task::none()
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
                Task::none()
            }
            Message::ShuffleList => {
                use rand::{ rng, seq::SliceRandom };

                let Viewing::Playlist(Some(list)) = self.viewing else {
                    return Task::none();
                };
                let tracks = self.playlists.get_playlist(list).unwrap()
                    .tracks
                    .iter()
                    .map(|track| match track {
                        PlaylistTrack::Track(id, _) => Some(*id),
                        _ => None,
                    })
                    .flatten()
                    .collect::<Vec<_>>();
                self.queue.resize(tracks.len(), 0);
                let mut shuffle = (0..tracks.len())
                    .collect::<Vec<usize>>();
                shuffle.shuffle(&mut rng());
                for (i, j) in shuffle.into_iter().enumerate() {
                    self.queue[i] = tracks[j];
                }
                self.stop();
                self.play_next();
                Task::none()
            }
            Message::SkipBack => {
                let Some(playing) = &self.playing else {
                    return Task::none();
                };
                match self.repeat {
                    RepeatStatus::None | RepeatStatus::One => {
                        self.sink.try_seek(Duration::from_secs(0));
                    }
                    RepeatStatus::All => {
                        let Some((current, _)) = &self.track_duration else {
                            return Task::none();
                        };
                        if current.as_secs() <= 1 && !self.queue.is_empty() {
                            self.queue.insert(0, track_hash(playing));
                            let last = unsafe {
                                self.queue.last().unwrap_unchecked()
                            };
                            self.queue.insert(0, *last);
                            self.play_next();
                        } else {
                            self.sink.try_seek(Duration::from_secs(0));
                        }
                    }
                }
                Task::none()
            }
            Message::SkipForward => {
                let Some(playing) = &self.playing else {
                    return Task::none();
                };
                match self.repeat {
                    RepeatStatus::All => {
                        self.queue.push(track_hash(playing));
                    }
                    _ => (),
                }
                self.stop();
                self.play_next();
                Task::none()
            }
            Message::StartScreen(msg) => {
                if let Some(start) = &mut self.start_screen {
                    if let Some(_) = &start.lib {
                        Task::done(Message::ScanDone)
                    } else {
                        start.update(msg)
                            .map(|s_msg| Message::StartScreen(s_msg))
                    }
                } else {
                    Task::none()
                }
            }
            Message::Stop => {
                self.play_status = PlayStatus::Stopped;
                self.queue.clear();
                self.stop();
                Task::none()
            }
            Message::ToggleMute => {
                self.mute = !self.mute;
                if self.mute {
                    self.sink.set_volume(0.0);
                } else {
                    self.sink.set_volume(self.volume);
                }
                Task::none()
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
                Task::none()
            },
            Message::ToggleRepeat => {
                self.repeat = match self.repeat {
                    RepeatStatus::None => RepeatStatus::One,
                    RepeatStatus::One => RepeatStatus::All,
                    RepeatStatus::All => RepeatStatus::None,
                };
                Task::none()
            },
            Message::UpdateProgress => {
                self.update_progress();
                Task::none()
            }
            Message::ViewLibrary(id) => {
                self.library.set_current(id);
                self.viewing = Viewing::Library;
                self.new_playlist_menu = false;
                self.selecting_playlist = None;
                Task::none()
            }
            Message::ViewPlaylist(val) => {
                self.viewing = Viewing::Playlist(val);
                self.new_playlist_menu = false;
                self.selecting_playlist = None;
                Task::none()
            }
            Message::VolumeChanged(val) => {
                self.volume = val;
                self.sink.set_volume(val);
                Task::none()
            }
            _ => Task::none(),
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
            } else {
                self.playhead_position = 0.0;
            }
        } else {
            let Some(playing) = &self.playing else {
                return; // prevent race conditions
            };
            let sink_pos = self.sink.get_pos();
            let duration = playing.metadata.duration.unwrap();
            self.track_duration = Some((sink_pos, duration));
            if !self.seeking {
                self.playhead_position =
                    sink_pos.as_secs_f32() / duration.as_secs_f32();
            }
        }
    }

    pub fn progress_subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(Duration::from_millis(10))
            .map(|_| Message::UpdateProgress)
    }
}
