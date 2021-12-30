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

#[derive(Debug, Deserialize)]
pub struct Feed {
    pub channel: Channel,
}

#[derive(Debug,)]
pub struct Channel {
    pub name: String,
    pub link: String,
    pub videos: Vec<Video>,
}

#[derive(Debug, PartialEq)]
pub struct Video {
    pub title: String,
    pub link: String,
    pub pub_date: String,
}

impl<'de> Deserialize<'de> for Video {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {

        #[derive(Deserialize, Debug)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Title, Link, PubDate }

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

        const FIELDS: &[&str] = &["title", "link", "pub_date"];
        d.deserialize_struct("Video", FIELDS, VideoVisitor)
    }
}

impl<'de> Deserialize<'de> for Channel {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {

        #[derive(Deserialize, Debug)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Title, Link, Videos }

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
                let name = title.ok_or_else(|| de::Error::missing_field("title_channel"))?;
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

        const FIELDS: &[&str] = &["title", "link", "videos"];
        d.deserialize_struct("Channel", FIELDS, ChannelVisitor)
    }
}
