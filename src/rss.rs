use data::date_time::DateTime;
use serenity::{model::prelude::ChannelId, prelude::Context};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone, Default)]
pub struct Rss {
    pub cached_date_time: HashMap<String, DateTime>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item(data::rss::Item);

impl Item {
    pub fn date(&self) -> &data::date_time::DateTime {
        self.0.date()
    }

    pub async fn post(&self, channel: ChannelId, ctx: Arc<Context>) {
        let message = channel
            .send_message(&ctx.http, |message| {
                message.embed(|embed| {
                    if let Some(title) = self.0.title() {
                        embed.title(title);
                    }

                    if let Some(description) = self.0.description() {
                        embed.description(description);
                    }

                    if let Some(url) = self.0.url() {
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

impl From<data::rss::Item> for Item {
    fn from(item: data::rss::Item) -> Self {
        Self(item)
    }
}
