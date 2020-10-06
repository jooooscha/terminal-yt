#[allow(dead_code)]
#[derive(Clone)]
pub struct Config {
    pub show_empty_channels: bool,
    pub mark_on_open: bool,
    pub down_on_mark: bool,
    pub app_title: String,
}

impl Config {
    pub fn default() -> Self {
        Config {
            show_empty_channels: true,
            mark_on_open: true,
            down_on_mark: true,
            app_title: String::from("TYT"),
        }
    }
}
