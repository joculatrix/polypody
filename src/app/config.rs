use std::{ error::Error, fs::File, io::{Read, Write}, path::PathBuf };
use serde::{ Deserialize, Serialize };

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    pub library: Library,
    pub playlists: Playlists,
    pub misc: Misc,
}

impl Config {
    pub fn from_file(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        Ok(toml::from_str(&s)?)
    }

    pub fn file_path() -> Result<PathBuf, Box<dyn Error>> {
        let mut path = crate::exe_path()?;
        path.push("config.toml");
        Ok(path)
    }

    pub fn write_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let toml = toml::to_string_pretty(&self)?;
        let mut file = File::create(path)?;
        Ok(file.write_all(toml.as_bytes())?)
    }
}

#[derive(Deserialize, Serialize)]
pub struct Library {
    pub path: PathBuf,
    pub full_rescan_on_start: bool,
    pub pins: Vec<PathBuf>,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            path: "/".into(),
            full_rescan_on_start: false,
            pins: vec![],
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Playlists {
    pub pins: Vec<PathBuf>,
}

impl Default for Playlists {
    fn default() -> Self {
        Self { pins: vec![] }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Misc {
    pub default_volume: f32,
}

impl Default for Misc {
    fn default() -> Self {
        Self { default_volume: 0.5 }
    }
}
