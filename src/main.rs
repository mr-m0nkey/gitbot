#![feature(proc_macro_hygiene, decl_macro)]
use dotenv::dotenv;
use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::prelude::{Context, EventHandler};
use request::GitHubEvent;

#[macro_use]
extern crate rocket;
extern crate crypto;
extern crate hex;
extern crate serde_json as json;

use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};

use std::thread;

mod request;

#[group]
#[commands(ping)]
struct General;

use std::env;

struct Handler;

impl EventHandler for Handler {}

#[post("/", data = "<payload>")]
fn index(event: GitHubEvent, payload: request::SignedPayload) {
    let data = json::from_str::<json::Value>(&payload.0).unwrap();
    match event {
        GitHubEvent::Push => {
            handle_push(data);
        }

        _ => {

        }
    }
}

fn handle_push(data: json::Value) {
    let number_of_commits = data["commits"].as_array().unwrap().len();
    let pusher = data["pusher"]["name"].as_str().unwrap();
    let repository = data["repository"]["name"].as_str().unwrap();
    println!("{} pushed {} commit(s) to {}", pusher, number_of_commits, repository);
}

fn main() {
    dotenv().ok();

    // TODO ceeate empty (thread safe) message queue

    let server_thread = thread::spawn(|| {
        let server = rocket::ignite()
            // .manage() TODO manage message queue
            .mount("/webhook", routes![index])
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
