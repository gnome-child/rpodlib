# rpodlib

A library for interacting with old iPods written in Rust.

Currently under development. This could break your iTunesDB!

### Status
- USB
  - windows: usb detection is intact, needs refactoring though
  - linux: doesn't detect iPods yet

- iTunesDB file
  - iPod classic recognizes database
  - all records apart from various forms of mhod are implemented
  - can add/remove any implemented record
  - can read/write and hash entire db for click-wheel based iPods
  - can query each type of record we actually care about
  - basic yet robust easy to work with wrappers for,
    - playlists
    - tracks
    - podcasts
  - writes iTunesLock file

- ArtworkDB file
  - fleshing out iTunesDB before working on this, architecture needs to be finalized

- Transcode
  - automatically detects compatible formats
  - able to transcode almost any audio file to a compatible format (Symphonia/qaac)
  
- Tests
  - to run current tests,
    - place some audio files in the audio_files folder under rpod-core/tests/fixtures
    - have qaac installed if transcoding is required
    - optionally plug in an iPod classic (iTunesDB.bak will be made)
