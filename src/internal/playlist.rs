use std::path::PathBuf;

use serde::{ Deserialize, Serialize, Serializer };

pub struct Playlist {
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
    fn new(toml: TomlPlaylist) -> Self {
        let title = toml.title;
        let img = toml.img.map(|s| PathBuf::from(s));
        let mut tracks = Vec::with_capacity(toml.tracks.capacity());
        let mut unresolved = vec![];
        for track in toml.tracks {
            let track = PathBuf::from(track);
            if track.try_exists().is_ok_and(|x| x) {
                tracks.push((super::library::path_hash(&track), track));
            } else {
                unresolved.push(track);
            }
        }
        Self { title, img, tracks, unresolved }
    }

    pub fn serialize(self) -> Result<String, Box<dyn std::error::Error>> {
        let playlist = TomlPlaylist {
            title: self.title,
            img: self.img.map(|path| path.to_str().unwrap().to_owned()),
            tracks: self.tracks
                .into_iter()
                .map(|(_, path)| path.to_str().unwrap().to_owned())
                .collect(),
        };
        Ok(toml::to_string_pretty(&playlist)?)
    }
}
