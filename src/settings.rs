use std::fmt::Display;

use config::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum Error {
    #[error("discord token could not be found")]
    DiscordTokenNotFound,
    #[error("rss channel id could not be found")]
    RssChannelNotFound,
    #[error("the config directory could not be found")]
    DirectoryNotFound,
    #[error("the config directory could not be created")]
    DirectoryCreationFailed,
    #[error("the settings could not be serialized")]
    SerializationFailed,
    #[error("the settings file could not be written")]
    WriteFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Token(String);

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Channel(serenity::model::prelude::ChannelId);

impl From<serenity::model::prelude::ChannelId> for Channel {
    fn from(channel_id: serenity::model::prelude::ChannelId) -> Self {
        Self(channel_id)
    }
}

impl From<Channel> for serenity::model::prelude::ChannelId {
    fn from(channel: Channel) -> Self {
        channel.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    token: Token,
    rss_channel: Channel,
    rss_list: Vec<String>,
    pub rss_refresh_seconds: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            token: Token::default(),
            rss_channel: Channel::default(),
            rss_list: vec![
                "https://blog.counter-strike.net/index.php/feed/".to_owned(),
                "https://blog.counter-strike.net/index.php/category/updates/feed/".to_owned(),
            ],
            rss_refresh_seconds: 600,
        }
    }
}

impl Settings {
    pub fn config_path() -> Result<std::path::PathBuf, Error> {
        let config_path = dirs_next::config_dir()
            .ok_or(Error::DirectoryNotFound)?
            .join("discord-bot-einar");

        if !config_path.exists() {
            std::fs::create_dir(config_path.as_path())
                .map_err(|_| Error::DirectoryCreationFailed)?;
        }

        Ok(config_path)
    }

    pub fn load() -> Result<Settings, Error> {
        let settings_path = Self::config_path()?.join("Settings.yaml");

        if !settings_path.exists() {
            let serialized = serde_yaml::to_string(&Settings::default())
                .map_err(|_| Error::SerializationFailed)?;

            std::fs::write(settings_path.clone(), serialized).map_err(|_| Error::WriteFailed)?;
        }

        let config = Config::builder()
            .add_source(config::File::from(settings_path))
            .build()
            .expect("settings file not found");
        let token = config
            .get::<Token>("token")
            .map_err(|_| Error::DiscordTokenNotFound)?;
        let rss_channel = config
            .get::<serenity::model::prelude::ChannelId>("rss_channel")
            .map_err(|_| Error::RssChannelNotFound)?
            .into();
        let rss_list = config
            .get_array("rss_list")
            .unwrap_or_default()
            .iter()
            .map(|v| v.to_string())
            .collect();
        let rss_refresh_seconds = config
            .get_string("rss_refresh_seconds")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60 * 10);

        Ok(Settings {
            token,
            rss_channel,
            rss_list,
            rss_refresh_seconds,
        })
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn rss_list(&self) -> &[String] {
        &self.rss_list
    }

    pub fn rss_channel(&self) -> Channel {
        self.rss_channel
    }

    pub fn rss_refresh_seconds(&self) -> u64 {
        self.rss_refresh_seconds
    }
}
