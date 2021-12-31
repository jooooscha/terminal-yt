use std::process::Command;
use crate::backend::config::Config;

pub fn notify_link(msg: &str) {
    send("Title", msg)
}

pub fn notify_open(video_title: &str) {
    send("Video Opening", video_title)
}

pub fn notify_error(e: &str) {
    send("Error", e)
}

fn send(title: &str, msg: &str) {
    // Command::new("notify-send").arg(title).arg(msg).output().expect("failed")
    let notifyer = Config::init().notify_with;
    let _ = Command::new(notifyer).arg(title).arg(msg).output().expect("failed");
}
    
