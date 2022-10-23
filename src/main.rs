mod rss;
mod settings;

use std::env;
use std::sync::Arc;
use std::time::Duration;

use itertools::Itertools;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::rss::Item;

struct RssCache;

impl TypeMapKey for RssCache {
    type Value = Arc<RwLock<rss::Rss>>;
}

struct SettingsCache;

impl TypeMapKey for SettingsCache {
    type Value = settings::Settings;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        log::info!("{} is connected", ready.user.name);

        if let Some(settings) = ctx.data.read().await.get::<SettingsCache>() {
            let ctx = Arc::new(ctx.clone());
            let rss_items = settings.rss_list().to_vec();
            let rss_channel = settings.rss_channel();
            let rss_refresh_seconds = settings.rss_refresh_seconds();

            tokio::spawn(async move {
                loop {
                    let rss_cache = ctx
                        .data
                        .read()
                        .await
                        .get::<RssCache>()
                        .expect("rss_cache not in typemap")
                        .clone();

                    for path in rss_items.iter() {
                        let items = data::rss::feed(path).await.unwrap_or_default();

                        if let Some(item) = items.first() {
                            // Get cached `DateTime` for item.
                            let mut rss_cache_write = rss_cache.write().await;
                            let cached_date_time = rss_cache_write
                                .cached_date_time
                                .entry(path.to_string())
                                .or_insert(*item.date());

                            // Filter away old posts.
                            let items = items
                                .into_iter()
                                .filter(|item| item.date() > cached_date_time)
                                .map(Item::from)
                                .sorted()
                                .collect::<Vec<_>>();

                            for item in &items {
                                item.post(rss_channel.into(), Arc::clone(&ctx)).await;
                            }

                            // Update cache with latest date if we have any.
                            if let Some(item) = items.first() {
                                *cached_date_time = *item.date();
                            }
                        }
                    }

                    tokio::time::sleep(Duration::from_secs(rss_refresh_seconds)).await;
                }
            });
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("!ping") {
            if let Err(why) = msg.channel_id.say(&ctx.http, "pong!").await {
                log::error!("error sending message: {:?}", why);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    SettingsError(#[from] settings::Error),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_logger();

    let settings = settings::Settings::load()?;
    let token = settings.token();

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token.to_string(), intents)
        .event_handler(Handler)
        .await
        .expect("error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<RssCache>(Default::default());
        data.insert::<SettingsCache>(settings);
    }

    if let Err(why) = client.start().await {
        log::error!("client error: {:?}", why);
    }

    Ok(())
}

fn setup_logger() {
    if env::var_os("RUST_LOG").is_none() {
        pretty_env_logger::formatted_builder()
            .filter_level(log::LevelFilter::Warn)
            .init();
    } else {
        pretty_env_logger::init();
    }
}
