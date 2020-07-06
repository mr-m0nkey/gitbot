#![feature(proc_macro_hygiene, decl_macro)]
use dotenv::dotenv;
use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::prelude::{Context, EventHandler};

#[macro_use]
extern crate rocket;

use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};

use std::thread;
use std::time::Duration;

#[group]
#[commands(ping)]
struct General;

use std::env;

struct Handler;

impl EventHandler for Handler {}


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() {
    dotenv().ok();

    // TODO create empty (thread safe) message queue



    let server_thread = thread::spawn(|| {
        let server = rocket::ignite()
            // .manage() TODO manage message queue
            .mount("/", routes![index])
            .launch();
    });

    let discord_thread = thread::spawn(|| {
        let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

        // Login with a bot token from the environment
        let mut client = Client::new(discord_token, Handler).expect("Error creating client");
        client.with_framework(
            StandardFramework::new()
                .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
                .group(&GENERAL_GROUP),
        );

        // start listening for events by starting a single shard
        if let Err(why) = client.start() {
            println!("An error occurred while running the client: {:?}", why);
        }
    });

    server_thread.join().unwrap();
    discord_thread.join().unwrap();

}

#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!")?;

    Ok(())
}
