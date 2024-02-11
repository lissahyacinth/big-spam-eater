use chrono::{Duration, TimeZone, Utc};
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;
use std::string::ToString;
use std::sync::Arc;

use serenity::async_trait;
use serenity::builder::CreateMessage;
use serenity::model::channel::Message;
use serenity::model::event::MessageUpdateEvent;
use serenity::model::gateway::Ready;
use serenity::model::id::{ChannelId, UserId};
use serenity::model::prelude::GuildId;
use serenity::model::user::User;
use serenity::prelude::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

struct Handler;

const VAGUELY_OKAY_WEBSITES: [&str; 7] = [
    "github.com",
    "bitbucket.com",
    "stackoverflow.com",
    "pastebin.com",
    "kaggle.com",
    "mit.edu",
    "usc.edu",
];

const BOT_CHANNEL: u64 = 1091681853603324047;
const HONEY_POT_CHANNEL: u64 = 889466095810011137;
const SPAM_EATER_ID: u64 = 1091478027264868422;

struct UserJoinDate;

impl TypeMapKey for UserJoinDate {
    type Value = Arc<RwLock<HashMap<UserId, i64>>>;
}

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

fn is_new_user(timestamp: Option<i64>) -> bool {
    if let Some(time) = timestamp {
        let diff: Duration = Utc::now() - Utc.timestamp_millis_opt(time * 1_000).unwrap();
        diff.num_hours() <= 1
    } else {
        true
    }
}

async fn warn_user(ctx: &Context, channel_id: ChannelId, user: &User) -> serenity::Result<Message> {
    let warning: String = format_args!(
        "Hi {member}, please wait a while after joining before sharing links or mentioning people.",
        member = user.mention()
    )
    .to_string();
    channel_id
        .send_message(&ctx.http, CreateMessage::new().content(warning))
        .await
}

async fn log_actions(ctx: &Context, content: &str, author_name: &str) -> serenity::Result<Message> {
    ChannelId::from(BOT_CHANNEL)
        .send_message(
            &ctx.http,
            CreateMessage::new().content(format!(
                "Hey bot team! I found '{}' from {} suspicious, so I deleted it. :)",
                content, author_name
            )),
        )
        .await
}

async fn log_ban(ctx: &Context, author_name: &str) -> serenity::Result<Message> {
    ChannelId::from(BOT_CHANNEL)
        .send_message(
            &ctx.http,
            CreateMessage::new().content(format!(
                "Hey bot team! '{}' posted in THE CHANNEL, so I deleted them :)",
                author_name
            )),
        )
        .await
}

async fn delete_message(ctx: &Context, message: &Message) -> serenity::Result<()> {
    message.delete(&ctx.http).await
}

async fn update_user_join_date(ctx: &Context, user: &User, join_date: i64) -> anyhow::Result<()> {
    if get_user_join_date(ctx, user).await.is_some() {
        Ok(())
    } else {
        let counter_lock = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<UserJoinDate>()
                .expect("Expected UserJoinDate in TypeMap.")
                .clone()
        };
        {
            let mut counter = counter_lock.write().await;
            let _ = counter.entry(user.id).or_insert(join_date);
        }
        Ok(())
    }
}

async fn get_user_join_date(ctx: &Context, user: &User) -> Option<i64> {
    let counter_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<UserJoinDate>()
            .expect("Expected UserJoinDate in TypeMap.")
            .clone()
    };
    let user_date_info = counter_lock.read().await;
    user_date_info.get(&user.id).copied()
}

async fn ban_user(ctx: &Context, guild_id: &GuildId, user: &UserId) -> serenity::Result<()> {
    guild_id
        .ban_with_reason(&ctx.http, user, 7, "Spam Channel honeypoint")
        .await
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.channel_id != ChannelId::from(BOT_CHANNEL) {
            if msg.channel_id == ChannelId::from(HONEY_POT_CHANNEL)
                && msg.author.id != UserId::from(SPAM_EATER_ID)
            {
                delete_message(&ctx, &msg).await.unwrap();
                log_ban(&ctx, msg.author.name.as_str()).await.unwrap();
                ban_user(&ctx, &msg.guild_id.unwrap(), &msg.author.id)
                    .await
                    .unwrap();
            }
            match msg.member {
                None => {
                    println!("Couldn't find MemberInfo for {:?}", msg.author);
                }
                Some(ref member_info) => {
                    match update_user_join_date(
                        &ctx,
                        &msg.author,
                        member_info.joined_at.unwrap().unix_timestamp(),
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Couldn't update User Info due to {}", e);
                        }
                    }
                    if (is_suspicious_url(msg.content.as_str()) || has_mention(&msg))
                        && is_new_user(get_user_join_date(&ctx, &msg.author).await)
                    {
                        warn_user(&ctx, msg.channel_id, &msg.author).await.unwrap();
                        delete_message(&ctx, &msg).await.unwrap();
                        log_actions(&ctx, msg.content.as_str(), msg.author.name.as_str())
                            .await
                            .unwrap();
                    }
                }
            }
        }
    }

    async fn message_update(
        &self,
        ctx: Context,
        _old_if_available: Option<Message>,
        new: Option<Message>,
        _event: MessageUpdateEvent,
    ) {
        if let Some(updated_message) = new {
            let author = updated_message.author;
            if is_suspicious_url(updated_message.content.as_str())
                | updated_message.mention_everyone
                && is_new_user(get_user_join_date(&ctx, &author).await)
            {
                warn_user(&ctx, updated_message.channel_id, &author)
                    .await
                    .unwrap();
                ctx.http
                    .delete_message(
                        updated_message.channel_id,
                        updated_message.id,
                        Some("Updated message with banned content"),
                    )
                    .await
                    .unwrap();
                log_actions(&ctx, updated_message.content.as_str(), author.name.as_str())
                    .await
                    .unwrap();
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ChannelId::from(BOT_CHANNEL)
            .send_message(
                &ctx.http,
                CreateMessage::new().content("Hey bot team! I'm online!".to_string()),
            )
            .await
            .unwrap();
        println!("{} is connected!", ready.user.name);
    }
}

async fn start_health_check() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buffer = [0; 1024];

            // Read the HTTP request from the socket
            match socket.read(&mut buffer).await {
                Ok(_) => {
                    let request = String::from_utf8_lossy(&buffer);

                    // Check if the request line contains a GET request to /health_check
                    if request.starts_with("GET /health_check HTTP/1.1") {
                        let response = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
                        let _ = socket.write_all(response.as_bytes()).await;
                    }
                },
                Err(e) => eprintln!("Failed to read from socket: {:?}", e),
            }
        });
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

    {
        let mut data = client.data.write().await;
        data.insert::<UserJoinDate>(Arc::new(RwLock::new(HashMap::default())));
    }

    tokio::spawn(async {
        if let Err(e) = start_health_check().await {
            eprintln!("Health check service failed: {}", e);
        }
    });

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
