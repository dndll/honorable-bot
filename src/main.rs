use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Write},
    sync::Arc,
};

use command::{Command, Manager, TwitterCommand};
use discord::DiscordConfig;
use gecko::CoingeckoConfig;

use serde::{Deserialize, Serialize};

use tokio::sync::mpsc::{self, Receiver, Sender};
use twitter::TwitterConfig;

pub mod command;
pub mod discord;
pub mod gecko;
pub mod twitter;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub twitter: TwitterConfig,
    pub discord: DiscordConfig,
    pub coingecko: CoingeckoConfig,
}

impl Config {
    fn persist(&self) -> Result<(), anyhow::Error> {
        let mut file = OpenOptions::new()
            .append(false)
            .write(true)
            .create(true)
            .open("config.json")?;
        file.write_all(serde_json::to_string_pretty(self)?.as_bytes())?;

        Ok(())
    }
    fn read() -> Result<Arc<Config>, anyhow::Error> {
        let config = File::open("config.json")?;
        let reader = BufReader::new(config);
        Ok(Arc::new(serde_json::from_reader(reader)?))
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();
    let config = Config::read()?;

    let (tx, mut rx): (Sender<Command>, Receiver<Command>) = mpsc::channel(256);
    let (twitter_tx, twitter_rx) = mpsc::channel(64);
    let (discord_tx, discord_rx) = mpsc::channel(256);
    let (_coingecko_tx, coingecko_rx) = mpsc::channel(64);

    config
        .twitter
        .start_manager(Arc::clone(&config), twitter_rx, tx.clone());
    config
        .discord
        .start_manager(Arc::clone(&config), discord_rx, tx.clone());
    config
        .coingecko
        .start_manager(Arc::clone(&config), coingecko_rx, tx.clone());

    let _main: Result<(), anyhow::Error> = tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Twitter(c) => {
                    let _ = twitter_tx
                        .send(c)
                        .await
                        .map_err(|e| log::error!("Failed to send command {}", e));
                }
                Command::Discord(c) => {
                    let _ = discord_tx
                        .send(c)
                        .await
                        .map_err(|e| log::error!("Failed to send command {}", e));
                }
                Command::Coingecko(_) => {}
            }
        }
        Ok(())
    })
    .await
    .expect("Main command loop died");

    Ok(())
}
