use std::process::Command;
use crate::backend::io::config::Config;

pub fn notify_link(msg: &str) {
    send("Title", msg)
}

pub fn notify_open(video_title: &str) {
    send("Video Opening", video_title)
}

pub fn notify_error(error: &str) {
    send("Error", error)
}

fn send(title: &str, msg: &str) {
    if let Ok(config) = Config::read() {
        let notifyer = config.notify_with;
        let _ = Command::new(notifyer).arg(title).arg(msg).output();
    }
}
    
