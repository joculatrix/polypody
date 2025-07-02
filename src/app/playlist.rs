use std::{ collections::{hash_map, HashMap}, error::Error, path::PathBuf };

use serde::{ Deserialize, Serialize };

pub struct PlaylistMap {
    map: HashMap<u64, Playlist>,
}

impl PlaylistMap {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn add_playlist(&mut self, pl: Playlist) -> u64 {
        let id = xxhash_rust::xxh3::xxh3_64(pl.filename.as_bytes());
        self.map.insert(id, pl);
        id
    }

    pub fn get_playlist(&self, id: u64) -> Option<&Playlist> {
        self.map.get(&id)
    }

    pub fn playlists(&self) -> hash_map::Iter<'_, u64, Playlist> {
        self.map.iter()
    }
}

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
    title: String,
    img: Option<String>,
    tracks: Vec<String>,
}

impl Playlist {
    fn new(toml: TomlPlaylist, filename: String) -> Self {
        let title = toml.title;
        let img = toml.img.map(|s| PathBuf::from(s));
        let mut tracks = Vec::with_capacity(toml.tracks.capacity());
        for track in toml.tracks {
            let track = PathBuf::from(track);
            if track.try_exists().is_ok_and(|x| x) {
                tracks.push(PlaylistTrack::Track(
                    crate::internal::library::path_hash(&track),
                    track
                ));
            } else {
                tracks.push(PlaylistTrack::Unresolved(track));
            }
        }
        Self { filename, title, img, tracks }
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
            title: self.title.clone(),
            img: self.img.clone().map(|path| path.to_str().unwrap().to_owned()),
            tracks: self.tracks
                .clone()
                .into_iter()
                .map(|x| match x {
                    PlaylistTrack::Track(_, path) => path.to_str().unwrap().to_owned(),
                    PlaylistTrack::Unresolved(path) => path.to_str().unwrap().to_owned(),
                })
                .collect(),
        };
        Ok(toml::to_string_pretty(&playlist)?)
    }
}
