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
- Specify when channels are updated with the [update_on](#how-do-i-subscribe) tag
- Filter "empty" channels
- Combine several feed in one [Custom Channel](#how-do-i-subscribe)

[![Rust build & test](https://github.com/jooooscha/terminal-yt/actions/workflows/rust.yml/badge.svg)](https://github.com/jooooscha/terminal-yt/actions/workflows/rust.yml)

## Usage

| Action                                        | Button      |
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

| Name                | Default       | Type | Description                                                                                                          |
|---------------------|---------------|------|----------------------------------------------------------------------------------------------------------------------|
| show_empty_channels | true          | bool | Show channels that have 0 new unmarked videos                                                                        |
| mark_on_open        | true          | bool | Mark a video if opened                                                                                               |
| down_on_mark        | true          | bool | Move pointer one down if a video is marked                                                                           |
| app_title           | "TYT"         | str  | The title of the left box                                                                                            |
| update_at_start     | true          | bool | Fetch new videos at start                                                                                            |
| sort_channels       | AlphaNumeric  | enum | One of `AlphaNumeric` or `ByTag`                                                                                     |
| video_player        | "mpv"         | str  | Could also be [umpv](https://raw.githubusercontent.com/mpv-player/mpv/master/TOOLS/umpv), vlc, or any other program. |
| sort_videos         | UneenDate     | enum | Can be one of: `Date, Text, UnseenDate, UnseenText`                                                                  |
| notify_with         | "notify-send" | str  | Could also be `dunstify` for example                                                                                 |
| use_dearrow_titles  | false         | bool | Uses the dearrow api for Youtube videos                                                                              |

## How do I "Subscribe"

The videos are fetched from a list of urls that have to be provided in the ` ~/.config/tyt/subscriptions.yaml ` file.

You can either use one Youtube channel as one internal channel, or you can combine multiple Youtbe channels to one internal (custom) channel.
Custom channels are shown as one single entry in the channel list.
In the `subscription.yaml` file they are declared in a seperate list. They have the same fiels as normal channels, except that theyc an take more multiple urls, and must be provided with a name.

The url of the channel is always `https://www.youtube.com/feeds/videos.xml?channel_id=<channel-id>`. You can get the channel-id via sites like this one: https://commentpicker.com/youtube-channel-id.php

For example:

``` yaml
---
channels:
    - url: "https://www.youtube.com/feeds/videos.xml?channel_id=UCBa659QWEk1AI4Tg--mrJ2A" # feed url
      name: "Tom Scott" # optional
      tag: FAVORITE # optional
      update_on: [always]
      block_regex: "EXTREMELY FUNNY" # filter out all videos that match this regex. Matched on the original title, not the one provided by dearrow

    - url: ...

custom_channels:
    - urls:
        - "https://www.youtube.com/feeds/videos.xml?channel_id=UCBa659QWEk1AI4Tg--mrJ2A" # feed url
        - "..."
      name: "Tom Scott" # mandatory in custom channels!
      tag: FAVORITE # optional
      update_on: [weekend]
```

The list `update_on` accepts any of `mon, tue, wed, thu, fri, sat, sub, workday, weekend, always, never`.


## Installation

#### Standard

- clone repo and `cd terminal-yt`
- run `cargo run` or `cargo install --path .`

#### Using Nix

- place config in `~/.config/tyt/subscriptions.yaml`
- `nix run github:jooooscha/terminal-yt`

#### Arch Linux

With your favorite AUR helper:
```bash
paru -S terminal-yt-bin  # latest release
# or
paru -S terminal-yt-git  # following main
```
