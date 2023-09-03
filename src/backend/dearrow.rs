use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Response {
    titles: Vec<ResponseTitles>,
}

#[derive(Debug, Deserialize)]
struct ResponseTitles {
    title: String,
    // original: bool,
    votes: usize,
}

pub fn get_best_title(video_id: &str) -> Option<String> {
    let client = Client::builder().build().unwrap();
    let url = format!("https://sponsor.ajay.app/api/branding/?videoID={}", video_id);
    let resp: Response = client.get(url).send().ok()?.json().ok()?;

    let first = resp.titles.first()?;

    match first.votes {
        0 => None,
        _ => Some(first.title.clone()),
    }

}
