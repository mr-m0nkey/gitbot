#![feature(proc_macro_hygiene, decl_macro)]
use dotenv::dotenv;
use request::GitHubEvent;
use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::model::*;
use serenity::model::{event::ResumedEvent, gateway::Ready};
use serenity::prelude::*;
use serenity::prelude::{Context, EventHandler};
use std::hash::{Hash, Hasher};

#[macro_use]
extern crate rocket;
extern crate crypto;
extern crate hex;
extern crate serde_json as json;

use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};

use rocket::State;
use std::sync::{Arc, Mutex};
use std::thread;

use hey_listen::sync::{
    ParallelDispatcher as Dispatcher, ParallelDispatcherRequest as DispatcherRequest,
};

use white_rabbit::{DateResult, Duration, Scheduler, Utc};

mod request;

#[group]
#[commands(ping)]
struct General;

use std::env;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, context: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);

        let time: u64 = 4000;

        let scheduler = {
            let mut context = context.data.write();
            context
                .get_mut::<SchedulerKey>()
                .expect("Expected Scheduler.")
                .clone()
        };

        let dispatcher = {
            let mut context = context.data.write();
            context
                .get_mut::<DispatcherKey>()
                .expect("Expected Dispatcher.")
                .clone()
        };

        let http = context.http.clone();
        let mut queue = {
            let mut context = context.data.write();
            context
                .get_mut::<EventQueue>()
                .expect("Expected Scheduler.")
                .clone()
        };

        let mut scheduler = scheduler.write();

        scheduler.add_task_duration(Duration::milliseconds(time as i64), move |_| {
            println!("fefefefefe");
            let http = http.clone();

            match queue.events.lock().unwrap().pop() {
                Some(event) => {
                    println!("{:#?}", event);
                }

                None => {}
            }

            // We add a function to dispatch for a certain event.
            dispatcher
                .write()
                .add_fn(DispatchEvent::GitEvent, print_events());

            // We return that our date shall happen again, therefore we need
            // to tell when this shall be.
            DateResult::Repeat(Utc::now() + Duration::milliseconds(time as i64))
        });
    }

    fn resume(&self, context: Context, _: ResumedEvent) {
        println!("Resumed");
    }
}

fn print_events() -> Box<dyn Fn(&DispatchEvent) -> Option<DispatcherRequest> + Send + Sync> {
    Box::new(move |_| Some(DispatcherRequest::StopListening))
}

#[derive(Debug)]
struct EventQueue {
    events: Mutex<Vec<String>>,
}

impl TypeMapKey for EventQueue {
    type Value = Arc<EventQueue>;
}

#[derive(Clone)]
enum DispatchEvent {
    GitEvent,
}

impl PartialEq for DispatchEvent {
    fn eq(&self, other: &DispatchEvent) -> bool {
        match (self, other) {
            (DispatchEvent::GitEvent, DispatchEvent::GitEvent) => true,
        }
    }
}

impl Eq for DispatchEvent {}

impl Hash for DispatchEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DispatchEvent::GitEvent => {}
        }
    }
}

struct DispatcherKey;

impl TypeMapKey for DispatcherKey {
    type Value = Arc<RwLock<Dispatcher<DispatchEvent>>>;
}

struct SchedulerKey;
impl TypeMapKey for SchedulerKey {
    type Value = Arc<RwLock<Scheduler>>;
}

#[post("/", data = "<payload>")]
fn index(event: GitHubEvent, payload: request::SignedPayload, event_queue: State<Arc<EventQueue>>) {
    println!("{:#?}", event_queue);
    event_queue
        .events
        .lock()
        .unwrap()
        .push(String::from("dfefe")); //TODO modify
    let data = json::from_str::<json::Value>(&payload.0).unwrap();
    match event {
        GitHubEvent::Push => {
            handle_push(data);
        }

        _ => {}
    }
}

fn handle_push(data: json::Value) {
    let number_of_commits = data["commits"].as_array().unwrap().len();
    let pusher = data["pusher"]["name"].as_str().unwrap();
    let repository = data["repository"]["name"].as_str().unwrap();
    println!(
        "{} pushed {} commit(s) to {}",
        pusher, number_of_commits, repository
    );
}

fn main() {
    dotenv().ok();

    // TODO ceeate empty (thread safe) message queue

    let event_queue = Arc::new(EventQueue {
        events: Mutex::new(Vec::new()),
    });

    let server_queue = event_queue.clone();

    let discord_queue = event_queue.clone();

    let server_thread = thread::spawn(move || {
        let server = rocket::ignite()
            .manage(server_queue)
            .mount("/webhook", routes![index])
            .launch();
    });

    let discord_thread = thread::spawn(move || {
        let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

        // Login with a bot token from the environment
        let mut client = Client::new(discord_token, Handler).expect("Error creating client");
        client.with_framework(
            StandardFramework::new()
                .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
                .group(&GENERAL_GROUP),
        );

        {
            let mut data = client.data.write();
            data.insert::<EventQueue>(discord_queue);
        }

        {
            let mut data = client.data.write();

            let scheduler = Scheduler::new(2);
            let scheduler = Arc::new(RwLock::new(scheduler));

            let mut dispatcher: Dispatcher<DispatchEvent> = Dispatcher::default();

            dispatcher
                .num_threads(4)
                .expect("Could not construct threadpool");

            data.insert::<DispatcherKey>(Arc::new(RwLock::new(dispatcher)));
            data.insert::<SchedulerKey>(scheduler);
        }

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
