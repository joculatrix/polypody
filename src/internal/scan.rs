use super::*;

enum ScanResult {
    Track(Track),
    Image(PathBuf),
}

pub fn scan(path: PathBuf) -> Library {
    let mut lib = Library::new();
    let root = scan_dir(&mut lib, path).unwrap();
    lib.set_root(root);
    lib
}

fn scan_file(path: &PathBuf) -> Option<ScanResult> {
    let Some(extension) = path.extension() else { return None; };
    let extension = extension.to_str().unwrap();
    match extension {
        "flac" => Some(ScanResult::Track(scan_flac(path))),
        "mp3" => Some(ScanResult::Track(scan_mp3(path))),
        "ogg" => Some(ScanResult::Track(scan_vorbis(path))),
        "wav" | "wave" => Some(ScanResult::Track(scan_wav(path))),
        "jpg" | "jpeg" | "png" => Some(ScanResult::Image(path.to_owned())),
        _ => None,
    }
}

fn scan_flac(path: &PathBuf) -> Track {
    let reader = claxon::FlacReader::open_ext(
        path,
        claxon::FlacReaderOptions {
            metadata_only: true,
            ..claxon::FlacReaderOptions::default()
        }
    )
        .unwrap();
    
    let title = reader.get_tag("TITLE").next().map(|s| s.to_owned());
    let artists = reader.get_tag("ARTIST").map(|s| s.to_owned()).collect();
    let album = reader.get_tag("ALBUM").next().map(|s| s.to_owned());
    let discnum = reader.get_tag("DISCNUMBER")
        .next()
        .map(|s| s.split('/').nth(0).unwrap().parse::<usize>().unwrap_or(0));
    let num = reader.get_tag("TRACKNUMBER")
        .next()
        .map(|s| s.split('/').nth(0).unwrap().parse::<usize>().unwrap_or(0));
    let duration = {
        let stream_info = reader.streaminfo();
        Duration::from_secs(
            stream_info.samples.unwrap()
                / stream_info.sample_rate as u64
        )
    };    

    Track {
        path: path.to_owned(),
        audio_type: AudioType::Flac,
        metadata: Metadata { title, artists, album, discnum, num, duration: Some(duration) }
    }
}

fn scan_mp3(path: &PathBuf) -> Track {
    let metadata = read_id3(path)
        .unwrap_or(read_ape(path)
            .unwrap_or(Metadata::default()));

    let duration = mp3_duration::from_read(&mut File::open(path).unwrap()).ok();
    let metadata = Metadata { duration, ..metadata };

    Track { path: path.to_owned(), audio_type: AudioType::Mp3, metadata }
}

fn scan_vorbis(path: &PathBuf) -> Track {
    use lewton::inside_ogg::OggStreamReader;

    let sample_len = get_vorbis_duration(path);

    let stream = OggStreamReader::new(File::open(path).unwrap())
        .unwrap();

    let duration = sample_len
        .map(|samples|
            Duration::from_secs(samples as u64
                / stream.ident_hdr.audio_sample_rate as u64)
        );

    let mut title = None;
    let mut artists = vec![];
    let mut album = None;
    let mut discnum = None;
    let mut num = None;

    for (key, value) in stream.comment_hdr.comment_list {
        match key.as_str() {
            "TITLE" => {
                title = Some(value);
            }
            "ARTIST" => {
                artists.push(value);
            }
            "ALBUM" => {
                album = Some(value);
            }
            "DISCNUMBER" => {
                discnum = Some(value.split('/').nth(0).unwrap().parse::<usize>().unwrap_or(0))
            }
            "TRACKNUMBER" => {
                num = Some(value.split('/').nth(0).unwrap().parse::<usize>().unwrap_or(0))
            }
            _ => (),
        }
    }

    let metadata = Metadata { title, artists, album, discnum, num, duration };

    Track { path: path.to_owned(), audio_type: AudioType::Vorbis, metadata }
}

/// Scans an ogg/vorbis file for its full length, given in samples.
///
/// This function is based on the implementation of
/// `stb_vorbis_stream_length_in_samples()` from `stb_vorbis`.
fn get_vorbis_duration(path: &PathBuf) -> Option<u32> {
    let mut f = File::open(path).ok()?;
    let init_len = f.stream_len().ok()?;
    let offset = if init_len > 65536 {
        init_len - 65536
    } else {
        0
    };
    f.seek(std::io::SeekFrom::Start(offset)).ok()?;
    let mut buf = [0; 5];
    loop {
        f.read_exact(&mut buf[..1]).ok()?;
        if buf[0] != 0x4F {
            continue;
        }
        if f.stream_len().unwrap() < 27 {
            return None;
        }
        let maybe_header = &mut buf[1..];
        f.read_exact(maybe_header).ok()?;
        // header start is marked by "OggS", followed by a version byte,
        // which as of right now is mandated to be 0
        if buf != [ 0x4F, 0x67, 0x67, 0x53, 0x00 ] {
            f.seek(std::io::SeekFrom::Current(-4)).ok()?;
            continue;
        }
        let header_type = &mut buf[..1];
        f.read_exact(header_type).ok()?;
        // 0x04 = EOS (end of stream), final page
        if header_type[0] == 0x04 {
            let mut hi = [0; 4];
            let mut lo = [0; 4];
            f.read_exact(&mut lo).ok()?;
            f.read_exact(&mut hi).ok()?;
            if matches!(
                [&lo, &hi],
                [&[0xFF, 0xFF, 0xFF, 0xFF], &[0xFF, 0xFF, 0xFF, 0xFF]]
            ) {
                return None;
            }
            if hi != [0; 4] {
                lo = [0xFF, 0xFF, 0xFF, 0xFE];
            }
            return Some(u32::from_le_bytes(lo));
        } else {
            continue;
        }
    }
}

fn scan_wav(path: &PathBuf) -> Track {
    let mut metadata = read_id3(path)
        .unwrap_or(Metadata::default());

    if metadata.duration.is_none() {
        let reader = hound::WavReader::open(path).unwrap();
        let duration = Some(Duration::from_secs(
            reader.duration() as u64 / reader.spec().sample_rate as u64));
        metadata = Metadata { duration, ..metadata };
    }

    Track { path: path.to_owned(), audio_type: AudioType::Wav, metadata }
}

fn read_id3(path: &PathBuf) -> Option<Metadata> {
    use id3::{ Tag, TagLike };

    match Tag::read_from_path(path) {
        Ok(tag) => {
            let title = tag.title().map(|s| s.to_owned());
            let artists = tag.artists()
                .map(|v| v.into_iter().map(|s| s.to_owned()).collect())
                .unwrap_or(vec![]);
            let album = tag.album().map(|s| s.to_owned());
            let discnum = tag.disc().map(|n| n.try_into().unwrap_or(0));
            let num = tag.track().map(|n| n.try_into().unwrap_or(0));
            let duration = tag.track().map(|s| Duration::from_secs(s as u64));

            if title.is_none()
                && artists.is_empty()
                && album.is_none()
                && discnum.is_none()
                && num.is_none()
                && duration.is_none()
            {
                return None;
            }

            Some(Metadata { title, artists, album, discnum, num, duration })
        }
        Err(_) => None,
    }
}

fn read_ape(path: &PathBuf) -> Option<Metadata> {
    match ape::read_from_path(path) {
        Ok(tag) => {
            let title = tag.item("title")
                .map(|i| i.to_owned().try_into().unwrap());
            let artists = tag.item("artist")
                .map(|i| i.to_owned().try_into().unwrap())
                .unwrap_or(vec![]);
            let album = tag.item("album")
                .map(|i| i.to_owned().try_into().unwrap());
            let num = tag.item("track")
                .map(|i|
                    <ape::Item as TryInto<String>>::try_into(i.to_owned())
                        .unwrap()
                        .split('/')
                        .nth(0)
                        .unwrap()
                        .parse::<usize>()
                        .unwrap_or(0)
                );

            Some(Metadata { title, artists, album, discnum: None, num, duration: None })
        }
        Err(_) => None,
    }
}

fn scan_dir(lib: &mut Library, path: PathBuf) -> Option<u64> {
    let path = path.as_path();
    assert!(path.is_dir());

    let mut dir = Directory::new(path.to_owned());
    let mut tracks_temp = vec![];
    let mut imgs_temp = vec![];

    for entry in path.read_dir().unwrap().into_iter() {
        if let Err(e) = entry {
            match e.kind() {
                std::io::ErrorKind::PermissionDenied => todo!(),
                std::io::ErrorKind::TimedOut => todo!(),
                std::io::ErrorKind::NotSeekable => todo!(),
                std::io::ErrorKind::QuotaExceeded => todo!(),
                std::io::ErrorKind::FileTooLarge => todo!(),
                std::io::ErrorKind::ResourceBusy => todo!(),
                std::io::ErrorKind::Deadlock => todo!(),
                std::io::ErrorKind::CrossesDevices => todo!(),
                std::io::ErrorKind::Interrupted => todo!(),
                std::io::ErrorKind::OutOfMemory => todo!(),
                std::io::ErrorKind::Other => todo!(),
                _ => todo!(),
            }
        } else if let Ok(entry) = entry {
            match entry.file_type() {
                Ok(ft) => {
                    if ft.is_dir() {
                        scan_dir(lib, entry.path())
                            .inspect(|id|
                                dir.subdirs.push(*id)
                            );
                    } else {
                        match scan_file(&entry.path()) {
                            Some(ScanResult::Image(data)) => {
                                imgs_temp.push(data);
                            }
                            Some(ScanResult::Track(track)) => {
                                tracks_temp.push(track);
                            }
                            None => (),
                        }
                    }
                }
                Err(_) => (),
            }
        }
    }

    tracks_temp.sort_unstable_by_key(|track|
        (
            track.metadata.discnum,
            track.metadata.num,
            track.metadata.title.clone(),
            track.path.clone()
        ));
    dir.tracks = tracks_temp.into_iter()
        .map(|track| lib.add_track(track))
        .collect();

    dir.img = sort_images(imgs_temp, &dir.path);

    if dir.subdirs.is_empty() && dir.tracks.is_empty() {
        None
    } else {
        Some(lib.add_directory(dir))
    } 
}

fn sort_images(imgs: Vec<PathBuf>, dir_path: &PathBuf) -> Option<PathBuf> {
    if !imgs.is_empty() && imgs.len() != 1 {
        let mut first_alphabetical: Option<&PathBuf> = None;
        let mut matches_dir_name = None;
        let mut matches_cover = None;
        let mut matches_folder = None;
        let mut matches_front = None;
        for img in &imgs {
            if let Some(first) = &mut first_alphabetical {
                if img.to_str().unwrap() < first.to_str().unwrap() {
                    *first = img;
                }
            } else {
                first_alphabetical = Some(img);
            }

            let name = img.file_stem().unwrap().to_str().unwrap().to_lowercase();

            if name == dir_path.file_stem().unwrap()
                .to_str().unwrap().to_lowercase()
            {
                matches_dir_name = Some(img);
                break;
            }
            if name.contains("cover") {
                matches_cover = Some(img);
            }
            if name.contains("folder") {
                matches_folder = Some(img);
            }
            if name.contains("front") {
                matches_front = Some(img);
            }
        }
        Some(if let Some(img) = matches_dir_name {
            img.to_owned()
        } else if let Some(img) = matches_cover {
            img.to_owned()
        } else if let Some(img) = matches_folder {
            img.to_owned()
        } else if let Some(img) = matches_front {
            img.to_owned()
        } else {
            unsafe { first_alphabetical.unwrap_unchecked().to_owned() }
        })
    } else {
        imgs.get(0).cloned()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn vorbis_duration_is_correct() {
        let track = scan_vorbis(&PathBuf::from("test/example.ogg"));
        assert!(track.metadata.duration.is_some());
        let length = track.metadata.duration.unwrap().as_secs();
        assert_eq!(length, 75);
    }
}
