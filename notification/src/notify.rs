use std::process::Command;

pub fn notify_user(msg: &String) {
    let _ = Command::new("notify-send").arg(msg).output().expect("failed");
}
