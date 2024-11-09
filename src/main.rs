use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use crate::ask::respond_ask;
use crate::chunking::chunk_string;
use crate::clean_messages::clean_message;
use crate::request::answer_request;
use crate::roadmaps::{create_roadmap, is_message_roadmap_request};
use crate::spam_detection::classify_message_spam;
use crate::user_info::retrieve_user_context;
use dotenv::dotenv;
use openai::set_key;
use serenity::all::Mention;
use serenity::async_trait;
use serenity::builder::CreateMessage;
use serenity::model::channel::Message;
use serenity::model::event::MessageUpdateEvent;
use serenity::model::gateway::Ready;
use serenity::model::id::{ChannelId, UserId};
use serenity::prelude::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{error, info};
use user_info::{UserContext, UserJoinDate};

mod ask;
mod chunking;
mod clean_messages;
mod messaging;
mod request;
mod roadmaps;
mod spam_detection;
mod user_info;
mod utilities;

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

#[derive(Debug)]
enum MessageClassification {
    Normal,
    MaybeSpam,
    DefinitelySpam(String),
}

async fn is_message_suspicious(
    message: &Message,
    user_join_date: Option<i64>,
) -> MessageClassification {
    if (messaging::is_suspicious_url(message.content.as_str()) | message.mention_everyone)
        && messaging::is_new_user(user_join_date)
    {
        // TODO: Track the context of user messages
        match classify_message_spam(message.content.clone(), vec![]).await {
            Ok(classification) => {
                if classification.is_spam {
                    MessageClassification::DefinitelySpam(classification.reason)
                } else {
                    info!(
                        "Message ({}) hit filter, not considered suspicious due to {}",
                        clean_message(message.content.as_str()),
                        classification.reason
                    );
                    MessageClassification::Normal
                }
            }
            Err(_) => MessageClassification::MaybeSpam,
        }
    } else {
        MessageClassification::Normal
    }
}

async fn reply_chunked(
    ctx: &Context,
    user: Mention,
    channel_id: ChannelId,
    content: String,
) -> anyhow::Result<()> {
    let formatted_message = format!("Hi {}, \n {}", user, content);
    for chunk in chunk_string(formatted_message.as_str(), 1_950) {
        channel_id
            .send_message(&ctx.http, CreateMessage::new().content(chunk))
            .await?;
    }
    Ok(())
}

async fn handle_request(ctx: &Context, message: &Message) -> anyhow::Result<()> {
    let maybe_query_author = match message.content.to_lowercase().as_str() {
        "!request" => message
            .referenced_message
            .as_ref()
            .map(|msg| (msg.content.clone(), msg.author.clone())),
        _ => Some((
            message
                .content
                .replacen("!request", "", 1)
                .trim()
                .to_string(),
            message.author.clone(),
        )),
    };

    if let Some((query, author)) = maybe_query_author {
        if let Some(response) = answer_request(query).await? {
            reply_chunked(ctx, author.mention(), message.channel_id, response).await?;
        }
    }
    Ok(())
}

async fn handle_roadmap(ctx: &Context, message: &Message) -> anyhow::Result<()> {
    if is_message_roadmap_request(message.content.clone(), vec![])
        .await?
        .is_roadmap
    {
        let user_context = retrieve_user_context(ctx, message).await;
        let created_roadmap = create_roadmap(message.content.clone(), user_context).await?;
        reply_chunked(
            ctx,
            message.author.mention(),
            message.channel_id,
            created_roadmap.roadmap,
        )
        .await?;
    }
    Ok(())
}

async fn handle_ask(ctx: &Context, message: &Message) -> anyhow::Result<()> {
    reply_chunked(
        ctx,
        message.author.mention(),
        message.channel_id,
        "Don't ask to ask, just ask! \nhttps://dontasktoask.com/".to_string(),
    )
    .await
}
async fn handle_message(ctx: Context, message: Message) {
    match is_message_suspicious(
        &message,
        user_info::get_user_join_date(&ctx, &message.author).await,
    )
    .await
    {
        MessageClassification::Normal => {}
        MessageClassification::MaybeSpam => {
            info!(
                "Removing message - likely spam - {}",
                message.content.as_str()
            );
            messaging::remove_message_and_log(&ctx, message.clone())
                .await
                .unwrap()
        }
        MessageClassification::DefinitelySpam(reason) => {
            info!(
                "Removing message - definitely spam - {}",
                message.content.as_str()
            );
            messaging::remove_warn_timeout_and_log(&ctx, message.clone(), reason.as_str())
                .await
                .unwrap()
        }
    }
    if messaging::is_message_request(&message) {
        if let Err(e) = handle_request(&ctx, &message).await {
            error!("Failed to create reply due to {e}")
        }
    } else if messaging::message_discusses_roadmaps(&message) {
        if let Err(e) = handle_roadmap(&ctx, &message).await {
            error!("Failed to create Roadmap due to {e}")
        }
    } else if messaging::is_message_ask(&message) {
        if let Err(e) = handle_ask(&ctx, &message).await {
            error!("Failed to link dontask due to {e}")
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.channel_id != ChannelId::from(BOT_CHANNEL)
            && msg.author.id != UserId::from(SPAM_EATER_ID)
        {
            if msg.channel_id == ChannelId::from(HONEY_POT_CHANNEL) {
                info!("Received message in Honeypot channel - removing");
                messaging::delete_message(&ctx, &msg).await.unwrap();
                messaging::log_ban(&ctx, msg.author.name.as_str())
                    .await
                    .unwrap();
                messaging::ban_user(&ctx, &msg.guild_id.unwrap(), &msg.author.id)
                    .await
                    .unwrap();
            }
            user_info::update_user_context(&ctx, &msg).await;
            match msg.member {
                None => {
                    error!("Couldn't find MemberInfo for {:?}", msg.author);
                }
                Some(ref member_info) => {
                    user_info::update_user_join_date(
                        &ctx,
                        &msg.author,
                        member_info.joined_at.unwrap().unix_timestamp(),
                    )
                    .await;
                    handle_message(ctx, msg).await;
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
            handle_message(ctx, updated_message).await;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

async fn start_health_check() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
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
                }
                Err(e) => eprintln!("Failed to read from socket: {:?}", e),
            }
        });
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let openai_key = env::var("OPENAI_KEY").expect("Expected an OpenAI Key in the environment");
    set_key(openai_key);
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
        data.insert::<UserContext>(Arc::new(RwLock::new(HashMap::default())));
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
