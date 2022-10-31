use itertools::Itertools;
use serenity::{model::prelude::ChannelId, prelude::Context};
use std::{cmp::Ordering, collections::HashMap, sync::Arc};

use crate::date_time::DateTime;

#[derive(Debug, Clone, Default)]
pub struct Rss {
    pub cached_date_time: HashMap<String, DateTime>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("request error")]
    Request(#[from] reqwest::Error),
    #[error("rss error")]
    Rss(#[from] rss::Error),
    #[error("conversion error")]
    Conversion,
}

#[derive(Debug, Clone)]
pub struct Id(rss::Guid);

#[derive(Debug, Clone)]
pub struct Item {
    id: Id,
    url: Option<String>,
    title: Option<String>,
    description: Option<String>,
    date: DateTime,
}

impl Item {
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn date(&self) -> &DateTime {
        &self.date
    }

    pub async fn post(&self, channel: ChannelId, ctx: Arc<Context>) {
        let message = channel
            .send_message(&ctx.http, |message| {
                message.embed(|embed| {
                    if let Some(title) = self.title() {
                        embed.title(title);
                    }

                    if let Some(description) = self.description() {
                        embed.description(description);
                    }

                    if let Some(url) = self.url() {
                        embed.url(url);
                    }

                    embed
                })
            })
            .await;

        if let Err(why) = message {
            log::error!("error sending message: {:?}", why);
        };
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id.0 == other.id.0
    }
}

impl Eq for Item {}

impl PartialOrd for Item {
    fn partial_cmp(&self, b: &Item) -> Option<Ordering> {
        Some(b.date.cmp(&self.date))
    }
}

impl Ord for Item {
    fn cmp(&self, b: &Item) -> Ordering {
        b.date.cmp(&self.date)
    }
}

impl TryFrom<&rss::Item> for Item {
    type Error = Error;

    fn try_from(item: &rss::Item) -> Result<Self, Self::Error> {
        let item = item.clone();
        let title = item
            .title()
            .map(|s| html_escape::decode_html_entities(s).to_string());
        let description = item
            .description()
            .map(|s| html_escape::decode_html_entities(s).to_string());
        let date = item
            .pub_date()
            .and_then(DateTime::parse_rfc2822)
            .ok_or(Error::Conversion)?;
        let id = item
            .guid()
            .map(|gid| Id(gid.clone()))
            .ok_or(Error::Conversion)?;

        Ok(Self {
            id,
            url: item.link,
            title,
            description,
            date,
        })
    }
}

pub async fn feed(path: &str) -> Result<Vec<Item>, Error> {
    let content = reqwest::get(path).await?.bytes().await?;
    let channel = rss::Channel::read_from(&content[..])?;
    let items = channel
        .items()
        .iter()
        .filter_map(|i| Item::try_from(i).ok())
        .sorted()
        .collect();

    Ok(items)
}
