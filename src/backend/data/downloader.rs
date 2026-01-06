use std::{collections::HashSet, fs::File};
use threadpool::ThreadPool;
use crate::backend::{core::FetchState, Error};
use log::*;

use super::{channel::Channel, video::DownloadState, StateUpdate};
use std::sync::{
    mpsc::channel,
    mpsc::{Receiver, Sender, TryRecvError},
};


pub struct Downloader {
    downloaded_videos: HashSet<String>,
    thread_pool: ThreadPool,
    status_sender: Sender<StateUpdate>,
}


impl Downloader {

    pub fn new(status_sender: Sender<StateUpdate>) -> Self {

        let thread_pool = ThreadPool::new(2);

        let downloaded_videos = HashSet::new();

        Self {
            thread_pool,
            downloaded_videos,
            status_sender,
        }
    }

    pub fn sync_channel(&self, channel: Channel) {

        // TODO: self.status_sender.send();
        let channel_id = channel.id().clone();

        for video in channel.videos {
            let url = video.link().clone();
            debug!("Downloading video: {}", url);

            if !self.downloaded_videos.contains(&url) {

                let status_sender = self.status_sender.clone();
                let channel_id = channel_id.clone();
                self.thread_pool.execute(move || {
                    // let size = Self::download_thread();
                    let _res = status_sender.send(
                        StateUpdate::new(
                            channel_id,
                            FetchState::VideoState(url, DownloadState::Downloaded)
                        )
                    );
                })

            }

        }
    }

    // function running in downloading thread
    fn download_thread() -> Result<usize, Error>{

        // download file to tmp location

        // on success, rename to final location

        // update status

        Ok(0)
    }

}
