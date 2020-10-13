# terminal-yt (alpha)

A small newsboat-inspired terminal youtube viewer written in Rust.

Shows a list of youtube channels and videos that can be opened in mpv (currently [umpv](https://pastebin.com/eAs451QF) only).
Build with [tui](https://github.com/fdehau/tui-rs) and termion as backend.

## Current features

- Load urls from _~/.config/tyt/urls_
- config in _~/.config/tyt/config_
- mark/unmark videos as watched
- support for Atom and RSS (Atom only with youtube)

## Keys

|                                                         |             |
|---------------------------------------------------------|-------------|
| up                                                      | k, up       |
| down                                                    | j, down     |
| open video                                              | o           |
| open channel                                            | o,l,right   |
| back                                                    | esc,h,right |
| mark                                                    | m           |
| unmark                                                  | M           |
| update                                                  | r           |
| show/hide channels that have no new videso (see config) | t           |

## Config

The config file is placed at _~/.config/tyt/config_ and is written in the toml file format.

If no config file is found, a config file with all options and their default values is written at start.

| Name                | Default | Type | Description                                                         |
|---------------------|---------|------|---------------------------------------------------------------------|
| show_empty_channels | true    | bool | Show channels that have 0 new unmarked videos |
| mark_on_open        | true    | bool | Mark a video if opened                                              |
| down_on_mark        | true    | bool | Move pointer one down if a video is marked                          |
| app_title           | "TYT"   | str  | The title of the left box                                           |
| update_at_start     | true    | bool | Fetch new viedos at start                                           |
