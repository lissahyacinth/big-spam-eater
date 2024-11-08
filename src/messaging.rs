use crate::clean_messages::clean_message;
use crate::{BOT_CHANNEL, VAGUELY_OKAY_WEBSITES};
use chrono::{Duration, TimeZone, Utc};
use serenity::all::{
    ChannelId, Context, CreateMessage, GuildId, Mentionable, Message, Timestamp, User, UserId,
};

pub fn is_suspicious_url(path: &str) -> bool {
    path.contains("http")
        // Not in any of our approved websites
        && (!(VAGUELY_OKAY_WEBSITES
        .iter()
        .any(|website| path.contains(website))))
}

/// Was the user's account created in the last hour?
pub fn is_new_user(timestamp: Option<i64>) -> bool {
    if let Some(time) = timestamp {
        let diff: Duration = Utc::now() - Utc.timestamp_millis_opt(time * 1_000).unwrap();
        diff.num_hours() <= 1
    } else {
        true
    }
}

async fn warn_user_generic(
    ctx: &Context,
    channel_id: ChannelId,
    user: &User,
) -> serenity::Result<Message> {
    warn_user_with_message(
        ctx,
        channel_id,
        user,
        "please wait a while after joining before sharing links or mentioning people.".to_string(),
    )
    .await
}

async fn warn_user_with_reason(
    ctx: &Context,
    channel_id: ChannelId,
    user: &User,
    reason: &str,
) -> serenity::Result<Message> {
    warn_user_with_message(
        ctx,
        channel_id,
        user,
        format!(" - this was removed because the message is considered to be `{reason}`"),
    )
    .await
}

async fn warn_user_with_message(
    ctx: &Context,
    channel_id: ChannelId,
    user: &User,
    message: String,
) -> serenity::Result<Message> {
    let warning: String = format_args!(
        "Hi {member}, {message}",
        member = user.mention(),
        message = message
    )
    .to_string();
    channel_id
        .send_message(&ctx.http, CreateMessage::new().content(warning))
        .await
}

async fn log_actions(
    ctx: &Context,
    content: &str,
    author_name: &str,
    reason: Option<&str>,
    timeout: bool,
) -> serenity::Result<Message> {
    let formatted_reason = match reason {
        None => "".to_string(),
        Some(reason) => format!(" because `{}`", reason),
    };
    let actions_taken = if timeout {
        " and timed them out until tomorrow"
    } else {
        ""
    }
    .to_string();
    ChannelId::from(BOT_CHANNEL)
        .send_message(
            &ctx.http,
            CreateMessage::new().content(format!(
                "Hey bot team! I found '{}' from {} suspicious{}, so I deleted it{}. :)",
                clean_message(content),
                author_name,
                formatted_reason,
                actions_taken
            )),
        )
        .await
}

pub async fn log_ban(ctx: &Context, author_name: &str) -> serenity::Result<Message> {
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

pub async fn delete_message(ctx: &Context, message: &Message) -> serenity::Result<()> {
    message.delete(&ctx.http).await
}

pub async fn ban_user(ctx: &Context, guild_id: &GuildId, user: &UserId) -> serenity::Result<()> {
    guild_id
        .ban_with_reason(&ctx.http, user, 7, "Spam Channel honeypot")
        .await
}

async fn timeout_user(ctx: &Context, guild_id: &GuildId, user: &UserId) -> serenity::Result<()> {
    guild_id
        .member(ctx, user)
        .await?
        .disable_communication_until_datetime(
            ctx,
            Timestamp::from_unix_timestamp(
                Timestamp::now().unix_timestamp() + Duration::days(1).num_seconds(),
            )
            .unwrap(),
        )
        .await
}

pub async fn remove_message_and_log(ctx: &Context, message: Message) -> anyhow::Result<()> {
    warn_user_generic(ctx, message.channel_id, &message.author)
        .await
        .unwrap();
    ctx.http
        .delete_message(
            message.channel_id,
            message.id,
            Some("Updated message with banned content"),
        )
        .await
        .unwrap();
    log_actions(
        ctx,
        message.content.as_str(),
        message.author.name.as_str(),
        None,
        false,
    )
    .await?;
    Ok(())
}

pub async fn remove_warn_timeout_and_log(
    ctx: &Context,
    message: Message,
    reason: &str,
) -> anyhow::Result<()> {
    warn_user_with_reason(ctx, message.channel_id, &message.author, reason).await?;
    ctx.http
        .delete_message(
            message.channel_id,
            message.id,
            Some("Message with banned content"),
        )
        .await
        .unwrap();
    timeout_user(ctx, &message.guild_id.unwrap(), &message.author.id).await?;
    log_actions(
        ctx,
        message.content.as_str(),
        message.author.name.as_str(),
        Some(reason),
        true,
    )
    .await?;
    Ok(())
}

pub fn message_discusses_roadmaps(message: &Message) -> bool {
    message.content.to_lowercase().contains("roadmap")
        | message.content.to_lowercase().contains("road map")
}

pub fn is_message_request(message: &Message) -> bool {
    message.content.to_lowercase().starts_with("!request")
}

pub fn is_message_ask(message: &Message) -> bool {
    message.content.to_lowercase().starts_with("!ask")
}