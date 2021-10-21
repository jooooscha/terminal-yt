use std::process::{Command, Output};

pub fn notify_link(msg: &String) -> Output {
    send(&String::from("Title"), msg)
}

pub fn notify_open(video_title: &String) -> Output {
    send(&String::from("Video Opening"), video_title)
}

fn send(title: &String, msg: &String) -> Output {
    Command::new("notify-send").arg(title).arg(msg).output().expect("failed")
}
    
