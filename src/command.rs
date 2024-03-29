use std::sync::Arc;

use coingecko_tokio::Market;
use egg_mode::tweet::Tweet;
use serenity::prelude::TypeMapKey;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::gecko::RuleResult;
use crate::Config;

pub enum Command {
    Twitter(TwitterCommand),
    Discord(DiscordCommand),
    Coingecko(CoingeckoCommand),
}
pub enum TwitterCommand {
    AddTwitterSubscription(String),
}
pub enum DiscordCommand {
    SendTweet(Tweet),
    SendCoingeckoRuleResult(RuleResult),
    SendCoingeckoBase(Vec<Market>),
}
pub enum CoingeckoCommand {}
pub struct CommandSender(pub Sender<Command>);
impl TypeMapKey for CommandSender {
    type Value = Arc<CommandSender>;
}

pub trait Manager<T> {
    fn start_manager(&self, config_cloned: Arc<Config>, rx: Receiver<T>, tx: Sender<Command>);
}
