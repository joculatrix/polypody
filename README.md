![image](https://github.com/user-attachments/assets/d3ccdc5b-7ae4-4bf1-8c90-0e7baed512e2)

Polypody is a player for downloaded music focused on delivering a true alternative to the
features of modern music platforms while adhering to the existing organization of the
user's music library.

> [!NOTE]
> As of right now, Polypody is very early in development. Some functionality or settings
> might be accessible only through editing config files. Some bugs or errors might cause
> crashes or present little or no information via the GUI. Please read the user guide or
> submit an issue if a problem occurs.

## Getting Started

Please read the [user guide](doc/USER_GUIDE.md) for information on installing and using
the application.

## Philosophy

Many music players have a problem of adhering too strictly to metadata. Music is re-sorted
by artist and album. Audio files without metadata often become a _long_ list of "Unknown
Track - Unknown Artist", without regard for how a user may have tried to organize those
metadata-less tracks into different groups. That, to me, is a UX failure. It also imposes
an idea of correct organizational structure on the music library which may not align with
the user's idea of organization.

A _polypody_ is a type of fern which is often epiphytic -- it grows harmlessly on the branches
of other plants for support and access to sunlight. Similarly, Polypody aims to grow harmlessly
on the branches of the user's filesystem tree, using its existing structure for organization.
It also aims to remove clutter, such as informing the user certain information is "Unknown",
where simply omitting the missing information would suffice.


## Functionality Checklist

### Audio file support

Overview of supported file formats. Completed items without sub-items can be
assumed to have basic metadata functionality (in the case of MP3, via a reasonable
variety of common formats). Undone items on this list are a low priority
unless requested by someone with audio they'd like to listen to in those
formats.

* [ ] AAC
* [ ] AIFF
* [ ] ALAC
* [ ] AU
* [x] FLAC
* [x] MP3
* [x] Vorbis (.ogg)
* [x] WAV
  * [x] Basic support
  * [ ] RIFF INFO metadata
  * [ ] BWF extension

### Features

The following features are planned at the very least for consideration, if not
for definite implementation.

* [ ] Audio visualizer
* [ ] Lyrics
  * [ ] LRC parsing
  * [ ] Live scrolling
* [x] Playlists
* [ ] Search by title, filename, artist, album
* [ ] Themes
  * [ ] Additional defaults
  * [ ] Custom user theming

### Technical considerations

The following tasks represent a general development to-do list:

* Consider alternative audio output frameworks, such as `interflow`, which would more easily
  allow for non-Alsa Linux solutions.
* Make error-handling more robust, and error-reporting more informative user-friendly.
* Better organize the application code into separate sub-screens, rather than holding all
  state in the `App` struct.
