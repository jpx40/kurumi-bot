# Kurumi-bot
![Build](https://github.com/Asm-Rosie/adhd-helper-bot/actions/workflows/rust.yml/badge.svg)

## How to run
first get ur discord bot token

once you have it create in ur project root dir a file called "Secrets.toml" you place this in there 
```toml
DISCORD_TOKEN="replace with ur token"
APP_ID="your app id"
GITHUB_REPO="your github repo (e.g. Asm-Rosie/kurumi-bot)
GITHUB_TOKEN="your github token"
LOG_CHANNEL="channel id"
BOT_ID = "bot user id here"
```
after you have ensured that all these values are set start the discord bot with `cargo run`
