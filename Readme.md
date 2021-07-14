# Twideo: Simple Telegram Bot for downloading videos from Twitter

## Setting up your environment
 1. [Download Rust](http://rustup.rs/).
 2. Create a new bot using [@Botfather](https://t.me/botfather) to get a token in the format `123456789:blablabla`.
 3. Get an [twitter access token](https://developer.twitter.com/en/apply-for-access).
 4. Initialise the `TWITTER_BEARER_TOKEN` and `TELOXIDE_TOKEN` environmental variable to your token:
```bash
# Unix-like
$ export TELOXIDE_TOKEN=<Your token here>
$ export TWITTER_BEARER_TOKEN=<Your bearer token here>

# Windows
$ set TELOXIDE_TOKEN=<Your token here>
$ set TWITTER_BEARER_TOKEN=<Your bearer token here>

Of course you can create a `.env` file and set your environmental variables there.
```
5. Run `cargo run` and enjoy the life :)