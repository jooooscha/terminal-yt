# terminal-yt (alpha)

A small newsboat-inspired terminal youtube viewer written in Rust.

Shows a list of youtube channels and videos that can be opened in mpv (currently [umpv](https://pastebin.com/eAs451QF) only).
Build with [tui](https://github.com/fdehau/tui-rs) with termion as backend.

## Current features

- Load urls from ~/.config/tyt/urls
- config in ~/.config/tyt/config (not ready)
- mark videos as watched
- support for Atom and rss (not yet)

## Keys

|                                              |         |
|----------------------------------------------|---------|
| up                                           | k, up   |
| down                                         | j, down |
| open channel/video                           | o       |
| mark                                         | m       |
| unmark                                       | M       |
| redraw screen (to correct after screen size) | R       |
