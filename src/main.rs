use anyhow::bail;
use chrono::{Duration, TimeZone, Utc};
use std::env;
use std::string::ToString;
use dotenv::dotenv;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

const VAGUELY_OKAY_WEBSITES: [&'static str; 6] = [
    "github.com",
    "bitbucket.com",
    "stackoverflow.com",
    "pastebin.com",
    "mit.edu",
    "usc.edu",
];

fn is_suspicious_url(path: &str) -> bool {
    path.contains("http")
        // Not in any of our approved websites
        && (!(VAGUELY_OKAY_WEBSITES
            .iter()
            .any(|website| path.contains(website))))
}

fn has_mention(message: &Message) -> bool {
    message.mention_everyone
}

fn is_new_user(message: &Message) -> anyhow::Result<bool> {
    match &message.member {
        None => bail!("This bot only works within a Guild, so members must have member data."),
        Some(member_info) => match member_info.joined_at {
            None => bail!("Expected user to have JoinedAt data"),
            Some(join_date) => {
                let diff: Duration = Utc::now()
                    - Utc
                        .timestamp_millis_opt(join_date.unix_timestamp() * 1_000)
                        .unwrap();
               Ok(diff.num_hours() <= 1)
            }
        },
    }
}

async fn warn_user(ctx: &Context, message: &Message) -> serenity::Result<Message> {
    message.channel_id.send_message(&ctx.http, |m| {
        m.content(
            format_args!(
                "Hi {member}, please wait a while after joining before sharing links or mentioning people.",
                member=message.author.mention()
        ))
    }).await
}

async fn delete_message(ctx: &Context, message: &Message) -> serenity::Result<()> {
    message.delete(&ctx.http).await
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if is_suspicious_url(msg.content.as_str()) || has_mention(&msg) {
            match is_new_user(&msg) {
                Ok(from_new_user) => {
                    if from_new_user {
                        warn_user(&ctx, &msg).await.unwrap();
                        delete_message(&ctx, &msg).await.unwrap();
                    }
                }
                Err(e) => {
                    println!("Couldn't verify message due to {}", e.to_string())
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents =
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILDS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
