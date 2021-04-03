use std::{collections::HashMap, sync::Arc};

use crate::{
    command::{CoingeckoCommand, Command, DiscordCommand, Manager},
    Config,
};
use coingecko::SimplePriceReq;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CoingeckoConfig {
    pub sleep_time_secs: u64,
}

impl Manager<CoingeckoCommand> for CoingeckoConfig {
    fn start_manager(
        &self,
        config_cloned: Arc<Config>,
        _rx: Receiver<CoingeckoCommand>,
        tx: Sender<Command>,
    ) {
        log::info!("Starting coingecko manager");
        let _ = tokio::spawn(async move {
            let client = coingecko::Client::new(reqwest::Client::new());

            // loop and call list
            // delay for config length

            if let Ok(state) = client.coins_list().await {
                let coin_ids = state
                    .iter()
                    .map(|coin| coin.id.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                let req = SimplePriceReq::new(coin_ids, "usd".into())
                    .include_market_cap()
                    .include_24hr_vol()
                    .include_24hr_change()
                    .include_last_updated_at();
                let state = client.simple_price(req).await.unwrap(); //TODO remove me

                let _ = tx
                    .send(Command::Discord(DiscordCommand::SendCoingeckoBase(state)))
                    .await;

                loop {
                    if let Ok(new_state) = client.simple_price(req.clone()).await {
                        // Compare the states, if any condition to jump is met, send a message to discord
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(
                        config_cloned.coingecko.sleep_time_secs,
                    ))
                    .await;
                }
            }
        });
    }
}
