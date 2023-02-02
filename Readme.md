# Twideo: Simple Telegram Bot for downloading videos from Twitter

## Setting up your environment
 1. [Download Rust](http://rustup.rs/).
 2. Create a new bot using [@Botfather](https://t.me/botfather) to get a token in the format `123456789:blablabla`.
 3. Get an [twitter access token](https://developer.twitter.com/en/apply-for-access).
 4. Optional Step: Install [PostgreSQL](https://www.postgresql.org/download/) database
 5. initialize the `TWITTER_BEARER_TOKEN`, `TWITTER_BEARER_TOKEN2`(to handle too many requests per second), `TELOXIDE_TOKEN` and `DATABASE_URL`(optional) environmental variables:
```bash
# Unix-like
$ export TELOXIDE_TOKEN=<Your token here>
$ export TWITTER_BEARER_TOKEN=<Your bearer token here>
$ export TWITTER_BEARER_TOKEN2=<Your 2nd bearer token or just leave it blank>
$ export DATABASE_URL=<Your database url or ignore it>

# Windows
$ set TELOXIDE_TOKEN=<Your token here>
$ set TWITTER_BEARER_TOKEN=<Your bearer token here>
$ set TWITTER_BEARER_TOKEN2=<Your 2nd bearer token or just leave it blank>
$ set DATABASE_URL=<Your database url or ignore it>

You can rename `.env-template` file to `.env` and put your environmental variables there.
```
5. Run `cargo run` and enjoy the life :)

## Getting Started
Just copy the link of the tweet and send it to the bot, It will convert tweet to telegram message:

![example](https://user-images.githubusercontent.com/79907489/174974007-cfc58c13-08d5-4b3e-b6ed-9d797fc4fb86.gif)


This bot also supports Inline mode:

![inline-example](https://user-images.githubusercontent.com/79907489/174976466-95406e20-30d8-4014-b78b-e9bd51ce126c.gif)
