use std::fmt::Display;

use serenity::model::prelude::ChannelId;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    token: Token,
    channel: ChannelId,
    feed: Vec<String>,
    refresh_rate: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            token: Token::default(),
            channel: ChannelId::default(),
            feed: vec![
                "https://blog.counter-strike.net/index.php/feed/".to_owned(),
                "https://blog.counter-strike.net/index.php/category/updates/feed/".to_owned(),
            ],
            refresh_rate: 600,
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
        let channel = config
            .get::<ChannelId>("channel")
            .map_err(|_| Error::RssChannelNotFound)?;
        let feed = config
            .get_array("feed")
            .unwrap_or_default()
            .iter()
            .map(|v| v.to_string())
            .collect();
        let refresh_rate = config
            .get_string("refresh_rate")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60 * 10);

        Ok(Settings {
            token,
            channel,
            feed,
            refresh_rate,
        })
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn feed(&self) -> &[String] {
        &self.feed
    }

    pub fn channel(&self) -> ChannelId {
        self.channel
    }

    pub fn refresh_seconds(&self) -> u64 {
        self.refresh_rate
    }
}
