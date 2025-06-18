
## Philosophy

Many music players, including those that ship with popular operating systems,
have a problem - at least by default - of adhering too strictly to metadata.
Music is re-sorted by artist and album. Audio files without metadata often become
a _long_ list of "Unknown Track - Unknown Artist", without regard for how a user
may have tried to organize those metadata-less tracks into different groups.
That, to me, is a UX failure. It also imposes an idea of correct organizational
structure on the music library which may not align with the user's idea of
organization.

My solution:

Display the music library with the exact same structure as the
user's actual file system. People who organize their music a certain way,
especially if they have large, long-kept libraries, likely do so for a reason.
Users can easily locate music in the music player's browser the same way they
know how to locate it in the file structure they've constructed.

Prefer metadata for titles and artists, but use file names where needed, refrain
from cluttering the UI with useless "unknown" information, and keep things where
the user put them.

## Functionality Checklist

### Audio file support

Overview of supported file formats. Completed items without sub-items can be
assumed to have basic metadata functionality (in the case of MP3, via a reasonable
variety of common formats). Undone items on this list are a low priority
unless requested by someone with audio they'd like to listen to in those
formats.

- [ ] AAC
- [ ] AIFF
- [ ] ALAC
- [ ] AU
- [x] FLAC
- [x] MP3
- [x] Vorbis (.ogg)
- [x] WAV
  - [x] Basic support
  - [ ] RIFF INFO metadata
  - [ ] BWF extension

### Features

The following features are planned at the very least for consideration, if not
for definite implementation.

- [ ] Audio visualizer
- [ ] Lyrics
  - [ ] LRC parsing
  - [ ] Live scrolling
- [ ] Playlists
  - [ ] Cover images
- [ ] Search by title, filename, artist, album
- [ ] Themes
  - [ ] Additional defaults
  - [ ] Custom user theming
