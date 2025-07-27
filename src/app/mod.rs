use config::Config;
use iced::task::Task;
use playlist::{Playlist, PlaylistMap, PlaylistTrack};
pub use view::ICON_FONT_BYTES;
use view::{queue, sidebar, start_screen};

use super::*;
use crate::internal::library::path_hash;

mod config;
mod playlist;
mod view;

#[derive(Debug, Clone)]
pub enum Message {
    CancelCreatePlaylist,
    CloseAddToPlaylist,
    CreatePlaylist,
    DeletePlaylist(u64),
    ImgPathChanged(String),
    ImgSelected(Option<rfd::FileHandle>),
    None,
    OpenImgDialog,
    OpenNewPlaylist,
    PinAdd(PinKind, PathBuf),
    PlaylistPathChanged(String),
    PlaylistRemove(usize),
    PlaylistSelected(u64),
    PlaylistSwap(usize, usize),
    PlaylistTitleChanged(String),
    PlayNext,
    PlayheadMoved(f32),
    PlayheadReleased,
    Queue(queue::QueueMessage),
    ScanDone,
    SelectPlaylist(u64),
    SidebarMessage(sidebar::SidebarMessage),
    StartScreen(start_screen::Message),
    ToggleMute,
    TogglePlay,
    ToggleRepeat,
    UpdateProgress,
    ViewLibrary(u64),
    ViewLibraryRoot,
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

    config:    Config,
    library:   Library,
    playlists: PlaylistMap,
    viewing:   Viewing,

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
    sidebar: sidebar::Sidebar,

    selecting_playlist: Option<u64>,

    new_playlist_menu:  bool,
    new_playlist_title: String,
    new_playlist_path:  String,
    new_playlist_img:   String,
}

impl App {
    pub fn new(stream_handle: rodio::OutputStreamHandle) -> Self {
        let (config, library, start_screen) =
            match Config::from_file(Config::file_path().unwrap()) {
                Ok(config) => {
                    let lib_cache_path = Library::file_path();
                    if let Ok(path) = lib_cache_path
                        && path.exists()
                    {
                        let lib = {
                            let config_lib_path = config.library.path.clone();
                            Library::from_file(&path).map_or_else(
                                |_| internal::scan(&config_lib_path),
                                |lib| {
                                    if lib.root_directory().path
                                        == config_lib_path
                                        && !config.library.full_rescan_on_start
                                    {
                                        internal::partial_scan(
                                            &config_lib_path,
                                            lib,
                                        )
                                    } else {
                                        internal::scan(&config_lib_path)
                                    }
                                },
                            )
                        };
                        (config, lib, None)
                    } else {
                        (
                            config,
                            Library::new(),
                            Some(start_screen::StartScreen::new()),
                        )
                    }
                }
                Err(e) => {
                    eprintln!("Couldn't read config: {e}");
                    (
                        Config::default(),
                        Library::new(),
                        Some(start_screen::StartScreen::new()),
                    )
                }
            };

        let config = config.verify_pins(&library);

        let mut playlists = PlaylistMap::new();
        playlists.scan_playlists();

        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        let volume = config.misc.default_volume.clamp(0.0, 1.0);
        sink.set_volume(volume);

        let sidebar = sidebar::Sidebar::new(
            config
                .library
                .pins
                .iter()
                .map(|path| {
                    (
                        path_hash(path),
                        path.file_stem().unwrap().to_str().unwrap().to_owned(),
                    )
                })
                .collect(),
            config
                .playlists
                .pins
                .iter()
                .map(|path| {
                    let id = path_hash(path);
                    (id, playlists.get_playlist(id).unwrap().title.to_owned())
                })
                .collect(),
        );

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
            sidebar,
            start_screen,
            selecting_playlist: None,
            new_playlist_menu: false,
            new_playlist_title: String::new(),
            new_playlist_path: String::new(),
            new_playlist_img: String::new(),
        }
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
                    if path.exists() { Some(path) } else { None }
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
            Message::DeletePlaylist(id) => {
                let Some(pl) = self.playlists.remove_playlist(id) else {
                    return Task::none();
                };
                let path = pl.file_path().unwrap();
                std::fs::remove_file(&path);
                self.config.playlists.pins = self
                    .config
                    .playlists
                    .pins
                    .iter()
                    .filter(|p| p.to_str().unwrap() != pl.filename)
                    .map(|p| p.to_owned())
                    .collect();
                self.write_config()
            }
            Message::ImgPathChanged(s) => {
                self.new_playlist_img = s;
                Task::none()
            }
            Message::ImgSelected(fh) => {
                if let Some(fh) = fh {
                    self.new_playlist_img =
                        fh.path().to_str().unwrap().to_owned();
                }
                Task::none()
            }
            Message::None => Task::none(),
            Message::OpenImgDialog => Task::perform(
                rfd::AsyncFileDialog::new()
                    .add_filter("image", &["png", "jpg", "jpeg"])
                    .pick_file(),
                Message::ImgSelected,
            ),
            Message::OpenNewPlaylist => {
                self.new_playlist_title.clear();
                self.new_playlist_img.clear();
                self.new_playlist_path.clear();
                self.new_playlist_menu = true;
                Task::none()
            }
            Message::PinAdd(kind, path) => match kind {
                PinKind::Library => {
                    self.config.library.pins.push(path.clone());
                    Task::done(
                        sidebar::SidebarMessage::LibraryAppend(
                            path_hash(&path),
                            path.file_stem()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_owned(),
                        )
                        .into(),
                    )
                }
                PinKind::Playlist => {
                    self.config.playlists.pins.push(path.clone());
                    let id = path_hash(&path);
                    Task::done(
                        sidebar::SidebarMessage::PlaylistAppend(
                            path_hash(&path),
                            self.playlists
                                .get_playlist(id)
                                .unwrap()
                                .title
                                .clone(),
                        )
                        .into(),
                    )
                }
            },
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
            },
            Message::PlaylistSelected(pl_id) => unsafe {
                let track_id = self.selecting_playlist.unwrap_unchecked();
                let pl =
                    self.playlists.get_playlist_mut(pl_id).unwrap_unchecked();
                pl.tracks.push(PlaylistTrack::Track(
                    track_id,
                    self.library
                        .get_track(track_id)
                        .unwrap_unchecked()
                        .path
                        .clone(),
                ));
                self.selecting_playlist = None;
                pl.write_to_file();
                Task::none()
            },
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
            },
            Message::PlaylistTitleChanged(s) => {
                self.new_playlist_title = s;
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

                let seek_pos =
                    Duration::from_secs((val * duration.as_secs_f32()) as u64);
                self.sink.try_seek(seek_pos);
                Task::none()
            }
            Message::PlayheadReleased => {
                self.seeking = false;
                Task::none()
            }
            Message::PlayNext => {
                if self.queue.len() == 0 {
                    return Task::none();
                }
                let track = self.queue.remove(0);
                let track = self.library.get_track(track).unwrap();
                self.playing = Some(track.clone());
                self.playhead_position = 0.0;
                self.track_duration = track
                    .metadata
                    .duration
                    .as_ref()
                    .map(|total| (Duration::from_secs(0), *total));
                self.sink.stop();
                self.sink.append(internal::audio::AudioStream::new(
                    &track.path,
                    self.codec_registry,
                    self.probe,
                    track.metadata.duration.unwrap(),
                ));
                self.sink.play();
                self.play_status = PlayStatus::Play;
                Task::none()
            }
            Message::Queue(msg) => self.update_queue(msg),
            Message::ScanDone => unsafe {
                let start = self.start_screen.take().unwrap_unchecked();
                self.library = start.lib.unwrap_unchecked();
                let _ = self.library.write_to_file().inspect_err(|e| {
                    eprintln!("Problem caching library data: {e}")
                });
                self.config.library.path = start.path.into();
                self.write_config()
            },
            Message::SelectPlaylist(track_id) => {
                self.selecting_playlist = Some(track_id);
                Task::none()
            }
            Message::SidebarMessage(msg) => {
                self.sidebar.update(msg, &mut self.config);
                self.write_config()
            }
            Message::StartScreen(msg) => {
                if let Some(start) = &mut self.start_screen {
                    if start.lib.is_some() {
                        Task::done(Message::ScanDone)
                    } else {
                        start.update(msg).map(Message::StartScreen)
                    }
                } else {
                    Task::none()
                }
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
            }
            Message::ToggleRepeat => {
                self.repeat = match self.repeat {
                    RepeatStatus::None => RepeatStatus::One,
                    RepeatStatus::One => RepeatStatus::All,
                    RepeatStatus::All => RepeatStatus::None,
                };
                Task::none()
            }
            Message::UpdateProgress => self.update_progress(),
            Message::ViewLibrary(id) => {
                use iced::widget::scrollable;

                self.library.set_current(id);
                self.viewing = Viewing::Library;
                self.new_playlist_menu = false;
                self.selecting_playlist = None;
                scrollable::scroll_to(
                    scrollable::Id::new("library"),
                    scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                )
            }
            Message::ViewLibraryRoot => {
                Task::done(Message::ViewLibrary(self.library.root_dir))
            }
            Message::ViewPlaylist(val) => {
                use iced::widget::scrollable;

                self.viewing = Viewing::Playlist(val);
                self.new_playlist_menu = false;
                self.selecting_playlist = None;
                if val.is_some() {
                    scrollable::scroll_to(
                        scrollable::Id::new("playlist"),
                        scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                    )
                } else {
                    scrollable::scroll_to(
                        scrollable::Id::new("playlist_list"),
                        scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                    )
                }
            }
            Message::VolumeChanged(val) => {
                self.volume = val;
                self.sink.set_volume(val);
                Task::none()
            }
        }
    }

    fn update_progress(&mut self) -> Task<Message> {
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
                Task::done(Message::PlayNext)
            } else {
                self.playhead_position = 0.0;
                self.stop();
                Task::none()
            }
        } else {
            let Some(playing) = &self.playing else {
                return Task::none(); // prevent race conditions
            };
            let sink_pos = self.sink.get_pos();
            let duration = playing.metadata.duration.unwrap();
            self.track_duration = Some((sink_pos, duration));
            if !self.seeking {
                self.playhead_position =
                    sink_pos.as_secs_f32() / duration.as_secs_f32();
            }
            Task::none()
        }
    }

    pub fn progress_subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(Duration::from_millis(10))
            .map(|_| Message::UpdateProgress)
    }
}
