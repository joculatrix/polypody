use std::{
    collections::{HashMap, hash_map},
    error::Error,
    fmt::write,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use iced::widget::combo_box;
use serde::{Deserialize, Serialize};

pub struct PlaylistMap {
    map: HashMap<u64, Playlist>,
}

impl PlaylistMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add_playlist(&mut self, pl: Playlist) -> u64 {
        let id = xxhash_rust::xxh3::xxh3_64(pl.filename.as_bytes());
        self.map.insert(id, pl);
        id
    }

    pub fn get_playlist(&self, id: u64) -> Option<&Playlist> {
        self.map.get(&id)
    }

    pub fn get_playlist_mut(&mut self, id: u64) -> Option<&mut Playlist> {
        self.map.get_mut(&id)
    }

    pub fn playlists(&self) -> hash_map::Iter<'_, u64, Playlist> {
        self.map.iter()
    }

    pub fn remove_playlist(&mut self, id: u64) -> Option<Playlist> {
        self.map.remove(&id)
    }

    pub fn scan_playlists(&mut self) -> Result<(), Box<dyn Error>> {
        let mut path = crate::exe_path()?;
        path.push("playlists/");
        if !path.exists() {
            return Ok(());
        }
        for entry in path.read_dir().unwrap().into_iter() {
            if let Err(e) = entry {
                eprintln!("Error reading entry: {e}");
            } else if let Ok(entry) = entry {
                let path = entry.path();
                let Some(extension) = path.extension() else {
                    continue;
                };
                if extension.to_str().unwrap() == "toml" {
                    let mut file = File::open(&path)?;
                    let mut s = String::new();
                    file.read_to_string(&mut s)?;
                    let Ok(toml) = toml::from_str(&s) else {
                        continue;
                    };
                    let pl = Playlist::from_toml(
                        toml,
                        path.file_name().unwrap().to_str().unwrap().to_owned(),
                    );
                    self.add_playlist(pl);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Playlist {
    pub filename: String,
    pub title: String,
    pub img: Option<PathBuf>,
    pub tracks: Vec<PlaylistTrack>,
}

#[derive(Clone)]
pub enum PlaylistTrack {
    Track(u64, PathBuf),
    Unresolved(PathBuf),
}

#[derive(Deserialize, Serialize)]
struct TomlPlaylist {
    title:  String,
    img:    Option<String>,
    tracks: Vec<String>,
}

impl Playlist {
    pub fn new(
        title: String,
        filename: String,
        img: Option<PathBuf>,
        tracks: Vec<PlaylistTrack>,
    ) -> Self {
        Self {
            title,
            filename,
            img,
            tracks,
        }
    }

    pub fn from_toml(toml: TomlPlaylist, filename: String) -> Self {
        let title = toml.title;
        let img = toml.img.map(|s| PathBuf::from(s));
        let mut tracks = Vec::with_capacity(toml.tracks.capacity());
        for track in toml.tracks {
            let track = PathBuf::from(track);
            if track.try_exists().is_ok_and(|x| x) {
                tracks.push(PlaylistTrack::Track(
                    crate::internal::library::path_hash(&track),
                    track,
                ));
            } else {
                tracks.push(PlaylistTrack::Unresolved(track));
            }
        }
        Self {
            filename,
            title,
            img,
            tracks,
        }
    }

    pub fn file_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        let mut path = crate::exe_path()?;
        path.push("playlists/");
        std::fs::create_dir_all(&path)?;
        path.push(self.filename.clone());
        Ok(path)
    }

    pub fn serialize(&self) -> Result<String, Box<dyn Error>> {
        let playlist = TomlPlaylist {
            title:  self.title.clone(),
            img:    self
                .img
                .clone()
                .map(|path| path.to_str().unwrap().to_owned()),
            tracks: self
                .tracks
                .clone()
                .into_iter()
                .map(|x| match x {
                    PlaylistTrack::Track(_, path) => {
                        path.to_str().unwrap().to_owned()
                    }
                    PlaylistTrack::Unresolved(path) => {
                        path.to_str().unwrap().to_owned()
                    }
                })
                .collect(),
        };
        Ok(toml::to_string_pretty(&playlist)?)
    }

    pub fn write_to_file(&self) -> Result<(), Box<dyn Error>> {
        let mut f = File::create(self.file_path()?)?;
        let toml = self.serialize()?;
        Ok(f.write_all(toml.as_bytes())?)
    }
}

impl std::fmt::Display for Playlist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}
