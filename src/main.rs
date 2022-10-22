mod rss;
mod settings;

use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use itertools::Itertools;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::rss::Item;
use settings::Settings;

struct DateTimeCache;

impl TypeMapKey for DateTimeCache {
    type Value = Arc<RwLock<HashMap<String, data::date_time::DateTime>>>;
}

struct Handler {
    settings: Settings,
    is_fetching_rss: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        log::info!("{} is connected", ready.user.name);

        if self.is_fetching_rss.load(Ordering::Relaxed) {
            return;
        }

        let ctx = Arc::new(ctx);
        let rss_items = self.settings.rss_list().to_vec();
        let rss_channel = self.settings.rss_channel();
        let rss_refresh_seconds = self.settings.rss_refresh_seconds();

        tokio::spawn(async move {
            loop {
                let cache_lock = ctx
                    .data
                    .read()
                    .await
                    .get::<DateTimeCache>()
                    .expect("expected date_time_cache in type_map")
                    .clone();

                for path in rss_items.iter() {
                    let items = data::rss::feed(path).await.unwrap_or_default();

                    if let Some(item) = items.first() {
                        // Ensure we have cached date.
                        let mut cache = cache_lock.write().await;
                        let cached_date = cache.entry(path.to_string()).or_insert(*item.date());

                        // Filter away old posts.
                        let items = items
                            .into_iter()
                            .filter(|item| item.date() > cached_date)
                            .map(Item::from)
                            .sorted()
                            .collect::<Vec<_>>();

                        for item in &items {
                            item.post(rss_channel, Arc::clone(&ctx)).await;
                        }

                        // Update cache with latest date if we have any.
                        if let Some(item) = items.first() {
                            *cached_date = *item.date();
                        }
                    }
                }

                tokio::time::sleep(Duration::from_secs(rss_refresh_seconds)).await;
            }
        });

        self.is_fetching_rss.swap(true, Ordering::Relaxed);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("!ping") {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                log::error!("error sending message: {:?}", why);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    setup_logger();

    let settings = Settings::load();
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&settings.token().to_string(), intents)
        .event_handler(Handler {
            settings,
            is_fetching_rss: AtomicBool::new(false),
        })
        .await
        .expect("error creating client");

    {
        // Build initial DateTimeCache used to keep track of latest RSS post.
        let mut data = client.data.write().await;
        data.insert::<DateTimeCache>(Arc::new(RwLock::new(HashMap::default())));
    }

    if let Err(why) = client.start().await {
        log::error!("client error: {:?}", why);
    }
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
