# User Guide
> [!IMPORTANT]
> Please note that Polypody is still early in its development. Some bugs might occur during regular use.
> Some features and settings for which there is not yet a GUI component may also temporarily require editing
> configuration files.

This guide intends to describe the process of downloading and setting up the application, as well as the
details of the application's behavior, in as much detail as possible for user reference.

* [Getting Started](#getting-started)
  * [Downloading a pre-built release](#downloading-a-pre-built-release)
  * [Building from source](#building-from-source)
  * [First start](#first-start)
* [User interface](#user-interface)
* [Configuration](#configuration)
* [Playlists](#playlists)
* [Library-scanning behavior](#library-scanning-behavior)
* [Errors](#errors)


## Getting Started

> [!NOTE]
> On Linux, Polypody currently requires ALSA/libasound.

### Downloading a pre-built release

Visit [this page](https://github.com/joculatrix/polypody/releases) for releases. Select the option that matches
your platform.

Polypody ships as a single executable file -- there's no need to run an installer. The only remaining part of
this process is that I recommend putting the executable in its own directory, as it will search that directory
for configuration files, playlist information, and other data that it generates during use.

### Building from source

If you have Rust and Cargo installed, building Polypody should be simple:

```
git clone https://github.com/joculatrix/polypody
cd polypody
cargo build --release
```

As with downloading a pre-built binary, I recommend moving Polypody (found in `path/to/polypody/target/release/`)
to its own directory to manage the configuration and data files it generates.

### First start

The first time you start Polypody, you should see a screen that looks similar to the following:

![image](https://github.com/user-attachments/assets/d7ecc228-927f-474a-b830-1c6ad5877dc5)

Polypody __focuses on preserving the organization of your existing music library__. As such, it takes a path to
a root directory your music is found under. This path will be saved and reused on subsequent startups.


## User Interface

Polypody's main interface looks like this:

![image](https://github.com/user-attachments/assets/2efc6346-2724-4f99-b53c-7698865bf03a)

From left to right:

* The __explorer sidebar__, by default, provides one button to view the root directory of your library, and one to
  view a list of all of your playlists. Underneath each of these sections, you can __pin__ specific folders and
  playlists for quick access. Once you've pinned a folder or playlist, hover over it on the sidebar to reveal buttons
  to reorder or unpin items.
* The __main view__ displays the folder or playlist you're inspecting (or the list of all of your playlists).
  * The header of a folder or playlist has the following buttons:
    * __Play:__ clear the current queue, and add all tracks in the folder/playlist to the queue, in order.
    * __Shuffle:__ clear the current queue, and add all tracks in the folder/playlist to the queue, in a random order.
    * __Pin__: pin the folder or playlist to the sidebar.
  * The main body of the window will show subdirectories (if viewing a folder), as well as all tracks in the folder/
    playlist. Hover over the tracks to reveal the following buttons (or you can right-click a track to play it, add it
    to the queue, or add it to a playlist):
    * __Play:__ clear the current queue and play the selected track immediately.
    * __Add to queue (+):__ add the track to the end of the queue.
    * __Move up/down (playlist only):__ reorder the tracks within the playlist.
    * __Delete (playlist only):__ remove the track from the playlist.
* The __queue__ lists every track that's been scheduled to play. Hover over the tracks to reveal buttons to reorder
  them or remove them from the queue.
* At the bottom of the window, the __control panel__ displays various controls with the following behaviors:
  * The __progress bar__ shows the total duration of the current track as well as the current position within the track.
    The playhead can be dragged to seek to different points within the track.
  * __Skip back:__ if the Repeat setting is "Off" or "Track", or if more than 1 second of the track has elapsed, the track
    is reset to the beginning. Otherwise, if the Repeat setting is "All" and less than 1 second of the track has elapsed,
    the track at the end the queue will play.
  * __Stop:__ immediately stop playing sound, and clear the queue.
  * __Play/Pause:__ self-descriptive.
  * __Shuffle:__ shuffle the order of the remaining tracks in the queue. (Note that in Polypody, shuffling is an _action_ you
    perform on the queue, rather than a toggled setting describing the audio as either "shuffled" or "not shuffled".)
  * __Skip forward:__ stop the current track, and play the next one in the queue, if any.
  * __Repeat:__ toggle between three settings:
    * __Off:__ everything in the queue plays only once.
    * __Track:__ the currently-playing track will restart once it finishes.
    * __All:__ once a track finishes, it will be appended to the end of the queue.
  * __Volume:__ drag the volume slider to change the volume of audio output. Click the speaker icon to toggle muting the audio.


## Configuration

Polypody stores user configuration options in a file at `path/to/polypody/config.toml`. Below is a demonstration of all
current configuration settings:

```toml
# /path/to/polypody/config.toml

[library]
path = "/path/to/music/library"
full_rescan_on_start = false    # forces the library to be rescanned entirely on every startup
pins = [
  "/path/to/directory",
  "/path/to/directory",
  ...
]

[playlists]
pins = [
  "filename_of_playlist.toml",
  "filename_of_playlist.toml",
  ...
]

[misc]
default_volume = 0.5            # initial value of volume slider on startup, from 0.0 - 1.0
```

Currently, `library.full_rescan_on_start` and `misc.default_volume` have no UI control, so the only way to edit these values is
by editing your config manually. After being set initially, `library.path` also can't currently be reset in-application without
editing the file.


## Playlists

Polypody stores playlists as configuration files in its own directory. Currently, the application has a new playlist menu (which
will silently fail without an error message when invalid input is given or when something goes wrong behind the scenes, which is
a problem hopefully soon to be fixed), and tracks can be reordered and deleted within the playlist, as well as added from the library.
However, once set, the image path can't be reset, nor the title or filename. For now, these must be edited within the playlist's
config file. 

For a playlist given a filename of `playlist`, a title of `Foo`, and an image path of `pictures/wahoo.png`, the configuration
file looks like the following:

```toml
# /path/to/polypody/playlists/playlist.toml

title = "Foo"
img = "pictures/wahoo.png"      # optional
tracks = [
  "/path/to/track",
  "/path/to/track",
  ...
]
```

When Polypody parses the playlist file, it skips any paths that don't exist as a valid path within the library.


## Library-scanning behavior

Polypody currently scans FLAC, MP3, OGG, and WAV files. Some of these files may have metadata Polypody can't recognize yet.
If every track in a folder has a "track number", the tracks are sorted by their number. Otherwise, they're sorted alphabetically.

A folder is omitted from the library if it contains no tracks and it has no subdirectories which contain tracks.

For images, Polypody recognizes PNG and JPG files. Polypody uses the names of the image files to select what it believes is the
most likely candidate for a cover image, with the following criteria (from highest to lowest priority):
* The image's file name matches the name of the directory it's in.
* The image's file name contains the word "cover".
* The image's file name contains the word "front".
* Otherwise, the image with the first file name alphabetically.

After your library has been scanned by the application, on subsequent startups, it will only perform a partial scan. During
a partial scan, tracks that have already been scanned into the library are ignored (and not updated), and if a directory
has already been given a cover image, the application won't check for a new one. Currently, until a button is added to the
user interface, the only ways to force a full rescan of the library are to delete the file at `path/to/polypody/.cache/library`,
remove the library path from your configuration file (see Configuration section), or change the config value `library.full_rescan_on_start`
to `true` before starting the application.


## Errors
Robust error-handling, and more helpful reporting of errors to the user, are on the to-do list for future work.
In the meantime (and after), if you encounter a crash or unexpected behavior, please submit an Issue or otherwise
contact me with as many details as possible regarding the problem. If you're able, launch the program in the
terminal and provide any error messages generated there.
