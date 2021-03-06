# Terminal-yt(BETA)

A small newsboat-inspired youtube viewer written in Rust.

Displays a list of youtube channels and videos that can be opened in mpv (currently [umpv](https://raw.githubusercontent.com/mpv-player/mpv/master/TOOLS/umpv) only).
Build with [tui](https://github.com/fdehau/tui-rs) and termion as backend.

![Screenshot](https://user-images.githubusercontent.com/32407/103141308-a290dd00-46c0-11eb-877a-fff1ad4f3b32.jpg)

## Features

- Fetch video from atom and rss feeds
- Open videos in a video player (per link only)
- Mark videos played
- Filter "empty" channels


## Usage

|                                               |             |
|-----------------------------------------------|-------------|
| up, down, left, right                         | j,k,h,l     |
| open video                                    | l,o,enter   |
| enter                                         | l,enter     |
| back                                          | esc,h,right |
| mark / unmark                                 | m,M         |
| update,fetch new videos                       | r           |
| show/hide channels that have no unseen videos | t           |
| copy video url                                | c           |


## Configuration

The config file is placed at ` ~/.config/tyt/config ` and is written in the toml file format.

If no config file is found, a config file with all options and their default values is written at start.

| Name                | Default | Type | Description                                         |
|---------------------|---------|------|-----------------------------------------------------|
| show_empty_channels | true    | bool | Show channels that have 0 new unmarked videos       |
| mark_on_open        | true    | bool | Mark a video if opened                              |
| down_on_mark        | true    | bool | Move pointer one down if a video is marked          |
| app_title           | "TYT"   | str  | The title of the left box                           |
| update_at_start     | true    | bool | Fetch new viedos at start                           |
| sort_by_tag         | false   | bool | Sort channels by tag or name                        |
| message_timeout     | 20      | u8   | Timeout for (error) messages                        |
| use_notify_send     | true    | bool | if `false` no message with notify-send will be send |

## Url file

The videos are fetched from a list of urls that have to be provided in the ` ~/.config/tyt/urls.yaml ` file.

For example:

``` yaml
---
channels:
    - url: "https://www.youtube.com/feeds/videos.xml?channel_id=UCBa659QWEk1AI4Tg--mrJ2A" # feed url
      name: "Tom Scott" # optional
      tag: FAVORITE # optional

    - url: ...

custom_channels:
    - urls:
        -"https://www.youtube.com/feeds/videos.xml?channel_id=UCBa659QWEk1AI4Tg--mrJ2A" # feed url
      name: "Tom Scott" # mandatory in custom channels!
      tag: FAVORITE # optional
```


## Installation

- clone repo and `cd terminal-yt`
- run `cargo build` or `cargo install --path .`
