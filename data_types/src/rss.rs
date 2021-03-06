use serde::{
    Deserialize,
    Deserializer,
    de::{
        self,
        Visitor,
        MapAccess,
    },
};
use std::fmt;
use chrono::DateTime;
use crate::internal;

#[derive(Debug, Deserialize)]
pub struct Feed {
    pub channel: Channel,
}

#[derive(Debug)]
pub struct Channel {
    /* #[serde(rename = "title")] */
    pub name: String,
    pub link: String,
    /* #[serde(rename = "item")] */
    pub videos: Vec<Video>,
}

#[derive(Debug, PartialEq)]
pub struct Video {
    pub title: String,
    pub link: String,
    /* #[serde(rename = "pubDate")] */
    pub pub_date: String,
}

#[allow(dead_code)]
impl Feed {
    pub fn to_internal_channel(self, original_link: &String) -> internal::Channel {
        let chan = self.channel;

        let name = chan.name;
        let link = chan.link;
        let videos = chan.videos.into_iter().map(|v| v.to_internal_video(original_link)).collect();

        internal::Channel {
            name,
            id: link,
            videos,
            ..internal::Channel::new()
        }
    }
}

#[allow(dead_code)]
impl Video {
    fn to_internal_video(self, channel_link: &String) -> internal::Video {
        // let title = self.title.first().unwrap().to_string();
        let title = self.title;
        let link = self.link;
        let pub_date = match DateTime::parse_from_rfc2822(&self.pub_date) {
            Ok(date) => date.to_rfc3339(),
            Err(e) => panic!("error parsing date in video {}: {}", title, e),
        };

        internal::Video {
            title,
            link,
            channel_link: channel_link.clone(),
            pub_date,
            ..internal::Video::new()
        }
    }
}
/*
 * pub fn from_str(xml: &String) -> Result<Feed, String> {
 *     let mut reader = Reader::from_str(xml);
 *     reader.trim_text(true);
 *
 *     let mut buf = Vec::new();
 *     let mut ns_buf = Vec::new();
 *
 *     let mut channel = Channel::new();
 *
 *     loop {
 *         match reader.read_namespaced_event(&mut buf, &mut ns_buf) {
 *             Ok((ref ns, Event::Start(ref e))) => {
 *                 match (*ns, e.local_name()) {
 *                     (Some(b"atom"), b"link") => println!("link {:?}", e.attributes().map(|a| a.unwrap().value).collect::<Vec<_>>()),
 *                     [> b"title" => println!("{}", e.unescape_and_decode(&reader).unwrap()), <]
 *                     _ => (),
 *                 }
 *             },
 *             [> Ok(Event::Text(e)) => <]
 *             Ok((_, Event::Eof)) => break,
 *             Err(e) => panic!("error at pos {}: {:?}", reader.buffer_position(), e),
 *             _ => (),
 *         }
 *     }
 *
 *     buf.clear();
 *
 *     Err(String::from("testing..."))
 * } */


/* struct VideoMapVisitor {}
 * impl VideoMapVisitor {
 *     fn new() -> Self {
 *         Self {}
 *     }
 * }
 *
 * impl<'de> Visitor<'de> for VideoMapVisitor {
 *     type Value = Video;
 *
 *     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
 *         formatter.write_str("struct Video")
 *     }
 *     fn visit_map<M: MapAccess<'de>>(self, mut access: M) -> Result<Self::Value, M::Error> {
 *         let mut item = Video {
 *             title: String::new(),
 *             link: String::new(),
 *             pub_date: String::new(),
 *         };
 *         while let Some((key, value)) = access.next_entry::<String, String>()? {
 *             item = Video {
 *                 title: key,
 *                 link: value,
 *                 pub_date: String::new(),
 *             };
 *         }
 *         Ok(item)
 *     }
 * }
 *
 * impl<'de> Deserialize<'de> for Video {
 *     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
 *         where D: Deserializer<'de>, {
 *             deserializer.deserialize_map(VideoMapVisitor::new())
 *     }
 * } */

impl<'de> Deserialize<'de> for Video {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {

        #[derive(Deserialize, Debug)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Title, Link, PubDate };

        struct VideoVisitor;

        impl<'de> Visitor<'de> for VideoVisitor {
            type Value = Video;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Video")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Video, V::Error> {
                let mut title = None;
                let mut link = None;
                let mut pub_date = None;
                while let Some(key) = match map.next_key::<String>() {
                    Ok(s) => s,
                    Err(e) => return Err(e),
                }{
                    match key.as_str() {
                        "title" => {
                            /* if title.is_some() {
                             *     return Err(de::Error::duplicate_field("title"));
                             * } */
                            title = Some(map.next_value()?);
                        }
                        "link" => {
                            if link.is_some() {
                                return Err(de::Error::duplicate_field("link"));
                            }
                            link = Some(map.next_value()?);
                        },
                        "pubDate" => {
                            if pub_date.is_some() {
                                return Err(de::Error::duplicate_field("pub_date"));
                            }
                            pub_date = Some(map.next_value()?);
                        },
                        _ => map.next_value()?,
                    }
                }
                let title = title.ok_or_else(|| de::Error::missing_field("title"))?;
                let link = link.ok_or_else(|| de::Error::missing_field("link"))?;
                let pub_date = pub_date.ok_or_else(|| de::Error::missing_field("pub_date"))?;

                let video = Video {
                    title,
                    link,
                    pub_date,
                };
                Ok(video)
            }
        }

        const FIELDS: &'static [&'static str] = &["title", "link", "pub_date"];
        d.deserialize_struct("Video", FIELDS, VideoVisitor)
    }
}

impl<'de> Deserialize<'de> for Channel {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {

        #[derive(Deserialize, Debug)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Title, Link, Videos };

        struct ChannelVisitor;

        impl<'de> Visitor<'de> for ChannelVisitor {
            type Value = Channel;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Channel")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Channel, V::Error> {
                let mut title = None;
                let mut link = None;
                let mut videos = Vec::new();
                while let Some(key) = match map.next_key::<String>() {
                    Ok(s) => s,
                    Err(e) => return Err(e),
                }{
                    match key.as_str() {
                        "title" => {
                            if title.is_some() {
                                return Err(de::Error::duplicate_field("title"));
                            }
                            title = Some(map.next_value()?);
                        },
                        "link" => {
                            if link.is_some() {
                                // nothing
                            }
                            link = Some(map.next_value()?);
                        },
                        "item" => {
                            videos.push(map.next_value()?);
                        },
                        _ => map.next_value()?,
                    }
                }
                let title = title.ok_or_else(|| de::Error::missing_field("title_channel"))?;
                let name = title;
                let link = link.ok_or_else(|| de::Error::missing_field("link_channel"))?;
                if videos.is_empty() {
                    return Err(de::Error::missing_field("videos_channel"));
                }

                let chan = Channel {
                    name,
                    link,
                    videos,
                };
                Ok(chan)
            }
        }

        const FIELDS: &'static [&'static str] = &["title", "link", "videos"];
        d.deserialize_struct("Channel", FIELDS, ChannelVisitor)
    }
}
