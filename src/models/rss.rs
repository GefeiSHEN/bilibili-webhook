use std::{error::Error, thread::sleep, time::Duration};
use log::{error, info, warn};
use quick_xml::de::from_str;
use reqwest::blocking::{self, Response};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Item {
    pub title: String,
    pub description: String,
    #[serde(rename = "pubDate", default)]
    pub pub_date: String,
    pub link: String,
    pub author: String,
}

#[derive(Deserialize)]
pub struct Channel {
    pub title: String,
    pub description: String,
    #[serde(rename = "lastBuildDate", default)]
    pub last_build_date: String,
    pub item: Vec<Item>,
}

#[derive(Deserialize)]
pub struct Rss {
    pub channel: Channel,
}

fn get(url: &str, mut retry: i8) -> Result<Response, reqwest::Error> {
    let response = blocking::get(url);
    match response {
        Ok(res) => Ok(res),
        Err(error) => {
            error!("Request failed, please check your configuration and network!");
            info!("Retrying {:?}", retry);
            error!("{:?}", error);
            if retry > 0 {
                let interval = 15;
                warn!("Retrying in {} seconds, attempt number {}", interval, 6 - retry);
                sleep(Duration::from_secs(interval));
                retry -= 1;
                get(url, retry)
            } else {
                error!("Failed to update source {}, giving up this update", url);
                Err(error)
            }
        }
    }
}

impl Rss {
    #[must_use]
    pub fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        let retry: i8 = 5;
        let res = match get(url, retry) {
            Ok(response) => response,
            Err(e) => {
                error!("Request failed: {}", e);
                return Err(e.into());
            },
        };

        let body = match res.text() {
            Ok(text) => text,
            Err(e) => {
                error!("Failed to parse response: {}", e);
                return Err(e.into());
            },
        };

        match from_str(&body) {
            Ok(rss) => Ok(rss),
            Err(e) => {
                error!("XML parsing failed: {}", e);
                error!("Response that failed parsing: {}", body);
                Err(e.into())
            },
        }
    }
}
