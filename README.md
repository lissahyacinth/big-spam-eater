# Eating Spam with Large Language Models

BigSpamEater is a Discord Bot that automatically finds and removes spam. 

## Honeypot
Discord bots target every single channel they can access. If you mark one as a honeypot and tell users not to post in it, then you can safely ban everyone who does.

## Classification Pipeline
The prompt used is around ~186 tokens. Assuming an average message size of 50 tokens, and a reply size of 20 tokens, we can work out the rough cost per message at 
(0.15 / 1_000_000 * 236) + (0.2 / 1_000_000 * 20) = $0.0000394 per message, or around 25,000 messages per $1 spent.

![Untitled-2024-07-09-1102](https://github.com/user-attachments/assets/2ddf46c7-4512-4e94-b5c2-80e4c04b7c54)

## Total Pricing
The machine picked is an EC2-Mini, and forms the majority of the hosting cost. You could likely drop this significantly by using spot pricing, but it currently works out to around $0.26 per day.

## Feature Creep
The bot also provides one-sentence answers to user queries upon request, but this feature was just for fun. 
