#![warn(rust_2018_idioms, missing_debug_implementations)]

mod bot;
mod config;

use bot::{Handler, COMMANDS_GROUP};
use config::Config;

use std::sync::Mutex;

use log::Level;
use serenity::{
    framework::StandardFramework,
    //prelude::GatewayIntents,
    Client,
};

use lazy_static::lazy_static;

const CONFIG_PATH: &str = "channels.json";

lazy_static! {
    static ref CONFIG: Mutex<Config> =
        Mutex::new(Config::from_file(CONFIG_PATH).unwrap_or_default());
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(Level::Warn).unwrap();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("|>"))
        .group(&COMMANDS_GROUP);

    // Code for version 0.11 of serenity
    // I'm using 0.10 because voice_state_update is broken in 0.11
    // But of course,
    // TODO: Update serenity to 0.11
    /*
    let intents = GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;
        */

    let token = std::env::var("DISCORD_TOKEN")
        .expect("expected environment variable DISCORD TOKEN to be defined");

    //let mut client = Client::builder(token, intents)
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client!");

    if let Err(why) = client.start().await {
        println!("An error occured while running the client! ({why:?})");
    }
}
