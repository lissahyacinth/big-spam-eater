Big Spam Eater is a moderation bot for Data Science Discord run by Lissa Hyacinth in the UK, who is the data controller. Source: github.com/lissahyacinth/big-spam-eater.

## What it reads
Your Discord user ID, username, server join time, and the text of messages in channels it can read. Nothing else.

## How it's used
• Messages from members who joined in the last hour that contain a link or @everyone are checked for spam. Spam is deleted, the author is timed out, and a copy is posted to a private mod channel.
• Anyone posting in the honeypot channel is banned.
• If you mention the bot, use its commands, or ask for a roadmap, your message is used to write a reply.

## Who else sees it
Message text being checked or answered is sent to OpenAI's API for processing (OpenAI privacy policy). It is not used to train models, sold, or shared with anyone else.

##How long it's kept
Recent messages and join times are held in memory only and are lost when the bot restarts. Server logs briefly contain the text of removed or answered messaged and are rotated automatically. Mod-channel copies stay in Discord until moderators delete them.

## Your rights
We rely on legitimate interests (keeping the server free of spam). To access, delete or object to your data, add an issue to this repo or DM a moderator. You can also complain to the ICO.
