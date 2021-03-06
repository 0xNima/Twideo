# Twideo: Simple Telegram Bot for downloading videos from Twitter

## Setting up your environment
 1. [Download Rust](http://rustup.rs/).
 2. Create a new bot using [@Botfather](https://t.me/botfather) to get a token in the format `123456789:blablabla`.
 3. Get an [twitter access token](https://developer.twitter.com/en/apply-for-access).
 4. Install [PostgreSQL](https://www.postgresql.org/download/) database
 5. Initialise the `TWITTER_BEARER_TOKEN`, `TELOXIDE_TOKEN` and `DATABASE_URL` environmental variable to your token:
```bash
# Unix-like
$ export TELOXIDE_TOKEN=<Your token here>
$ export TWITTER_BEARER_TOKEN=<Your bearer token here>
$ export DATABASE_URL=<Your database url>

# Windows
$ set TELOXIDE_TOKEN=<Your token here>
$ set TWITTER_BEARER_TOKEN=<Your bearer token here>
$ set DATABASE_URL=<Your database url>

Of course you can create a `.env` file and set your environmental variables there.
```
5. Run `cargo run` and enjoy the life :)

## Getting Started
Just copy the link of the tweet and send it to the bot, It will convert tweet to telegram message:

![example](https://user-images.githubusercontent.com/79907489/174974007-cfc58c13-08d5-4b3e-b6ed-9d797fc4fb86.gif)


This bot also supports Inline mode:

![inline-example](https://user-images.githubusercontent.com/79907489/174976466-95406e20-30d8-4014-b78b-e9bd51ce126c.gif)
