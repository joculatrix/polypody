use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
};

use xxhash_rust::xxh3::xxh3_64;

use super::{Directory, Track};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Library {
    pub root_dir: u64,
    pub curr_dir: u64,

    dir_registry:   HashMap<u64, Directory>,
    track_registry: HashMap<u64, Track>,
}

impl Library {
    pub fn new() -> Self {
        Self {
            root_dir: 0,
            curr_dir: 0,
            dir_registry: HashMap::new(),
            track_registry: HashMap::new(),
        }
    }

    pub fn add_directory(&mut self, dir: Directory) -> u64 {
        let hash = directory_hash(&dir);

        for subdir in &dir.subdirs {
            self.dir_registry
                .entry(*subdir)
                .and_modify(|v| v.parent = hash);
        }

        self.dir_registry.insert(hash, dir);

        hash
    }

    pub fn get_directory(&self, id: u64) -> Option<&Directory> {
        self.dir_registry.get(&id)
    }

    pub fn get_directory_mut(&mut self, id: u64) -> Option<&mut Directory> {
        self.dir_registry.get_mut(&id)
    }

    pub fn add_track(&mut self, track: Track) -> u64 {
        let hash = track_hash(&track);
        self.track_registry.insert(hash, track);
        hash
    }

    pub fn get_track(&self, id: u64) -> Option<&Track> {
        self.track_registry.get(&id)
    }

    pub fn current_directory(&self) -> &Directory {
        self.dir_registry.get(&self.curr_dir).unwrap()
    }

    pub fn root_directory(&self) -> &Directory {
        self.dir_registry.get(&self.root_dir).unwrap()
    }

    pub fn set_current(&mut self, id: u64) {
        self.curr_dir = id;
    }

    pub fn set_root(&mut self, id: u64) {
        self.root_dir = id;
        self.curr_dir = id;
    }

    pub fn file_path() -> std::io::Result<PathBuf> {
        let mut path = crate::exe_path()?;
        path.push(".cache/");
        std::fs::create_dir_all(&path)?;
        path.push("library");
        Ok(path)
    }

    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        Ok(bincode::serde::decode_from_reader(
            BufReader::new(File::open(path)?),
            bincode::config::standard(),
        )?)
    }

    pub fn write_to_file(&self) -> std::io::Result<()> {
        let mut f = File::create(Self::file_path()?)?;
        let data =
            bincode::serde::encode_to_vec(self, bincode::config::standard())
                .unwrap();
        f.write_all(&data)
    }
}

pub fn directory_hash(dir: &Directory) -> u64 {
    xxh3_64(dir.path.as_os_str().as_encoded_bytes())
}

pub fn path_hash(path: &PathBuf) -> u64 {
    xxh3_64(path.as_os_str().as_encoded_bytes())
}

pub fn track_hash(track: &Track) -> u64 {
    xxh3_64(track.path.as_os_str().as_encoded_bytes())
}
