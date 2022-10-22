use std::fmt::Display;

use config::Config;
use serenity::model::prelude::ChannelId;

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum Error {
    #[error("the config directory could not be found")]
    DirectoryNotFound,
    #[error("the config directory could not be created")]
    DirectoryCreationFailed,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Settings {
    token: Token,
    rss_channel: ChannelId,
    rss_list: Vec<String>,
    rss_refresh_seconds: u64,
}

impl Settings {
    pub fn load() -> Result<Settings, Error> {
        let mut config_path = dirs_next::config_dir().ok_or(Error::DirectoryNotFound)?;
        config_path.push("discord-bot-einar");

        if !config_path.exists() {
            std::fs::create_dir(config_path.as_path())
                .map_err(|_| Error::DirectoryCreationFailed)?;
        }

        config_path.push("Settings.yml");

        let config = Config::builder()
            .add_source(config::File::from(config_path))
            .build()
            .expect("settings file not found");

        let token = config
            .get_string("token")
            .expect("token not found in settings")
            .as_str()
            .into();
        let rss_channel = config
            .get_string("rss_channel")
            .expect("rss channel not found in settings")
            .parse::<u64>()
            .expect("error parsing rss channel to u64")
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

    pub fn rss_channel(&self) -> ChannelId {
        self.rss_channel
    }

    pub fn rss_refresh_seconds(&self) -> u64 {
        self.rss_refresh_seconds
    }
}
