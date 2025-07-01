use std::{error::Error, path::PathBuf};

use serde::{ Deserialize, Serialize };

pub struct Playlist {
    filename: String,
    title: String,
    img: Option<PathBuf>,
    tracks: Vec<(u64, PathBuf)>,
    unresolved: Vec<PathBuf>,
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
        let mut unresolved = vec![];
        for track in toml.tracks {
            let track = PathBuf::from(track);
            if track.try_exists().is_ok_and(|x| x) {
                tracks.push((crate::internal::library::path_hash(&track), track));
            } else {
                unresolved.push(track);
            }
        }
        Self { filename, title, img, tracks, unresolved }
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
                .map(|(_, path)| path.to_str().unwrap().to_owned())
                .collect(),
        };
        Ok(toml::to_string_pretty(&playlist)?)
    }
}
