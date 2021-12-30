# Terminal-yt

A small newsboat-inspired youtube subscription manager written in Rust.

Tyt can parse atom and RSS feeds and was written with video feed from YouTube or Twitch in mind.
The default player is mpv. However, this can be changed in the settings.

Tyt was build with [tui](https://github.com/fdehau/tui-rs) and termion as backend.

![Screenshot](https://user-images.githubusercontent.com/57965027/138331749-8eed019d-8825-459f-bd87-177a98eaf61b.png)

## Features

- Fetch video from atom and RSS feeds
- Open videos in a video player (per link)
- Mark videos played
- Filter "empty" channels
- Combine several feed in one _Custom Channel_

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

The config file is placed at ` ~/.config/tyt/config.yml ` and is written in the yml file format.

If no config file is found, a config file with all options and their default values is written at start.

| Name                | Default | Type | Description                                                                                                                     |
|---------------------|---------|------|---------------------------------------------------------------------------------------------------------------------------------|
| show_empty_channels | true    | bool | Show channels that have 0 new unmarked videos                                                                                   |
| mark_on_open        | true    | bool | Mark a video if opened                                                                                                          |
| down_on_mark        | true    | bool | Move pointer one down if a video is marked                                                                                      |
| app_title           | "TYT"   | str  | The title of the left box                                                                                                       |
| update_at_start     | true    | bool | Fetch new videos at start                                                                                                       |
| sort_by_tag         | false   | bool | Sort channels by tag or name                                                                                                    |
| message_timeout     | 20      | u8   | Timeout for (error) messages                                                                                                    |
| notify_with         | "notify-send    | str |  |
| video_player        | "mpv"     | str  | Can be changed to [umpv](https://raw.githubusercontent.com/mpv-player/mpv/master/TOOLS/umpv), vlc, or any other program.        |

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
