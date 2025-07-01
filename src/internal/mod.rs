use std::fs::File;
use std::io::{ Read, Seek };
use std::path::PathBuf;
use std::time::Duration;

pub use library::Library;
pub use scan::scan;

pub mod audio;
pub mod library;
pub mod scan;


#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Directory {
    pub parent: u64,
    pub path: PathBuf,
    pub img: Option<PathBuf>,
    pub subdirs: Vec<u64>,
    pub tracks: Vec<u64>,
}

impl Directory {
    pub fn new(path: PathBuf) -> Self {
        Self {
            parent: 0,
            path,
            img: None,
            subdirs: vec![],
            tracks: vec![],
        }
    }
}

#[derive(Copy, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum AudioType {
    Flac,
    Mp3,
    Vorbis,
    Wav,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Track {
    pub path: PathBuf,
    pub audio_type: AudioType,
    pub metadata: Metadata,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub discnum: Option<usize>,
    pub num: Option<usize>,
    pub duration: Option<Duration>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            title: None,
            artists: vec![],
            album: None,
            discnum: None,
            num: None,
            duration: None,
        }
    }
}
