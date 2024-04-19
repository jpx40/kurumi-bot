use poise::serenity_prelude as serenity;

mod commands;
mod db;
mod download_docs;
mod events;
mod handler;
mod rich_presence;
mod secrets;
mod utils;



use commands::{
    help::*,
    info::*,
    moderation::{
        ban::{ban, unban},
        kick::kick,
        mute::{mute, unmute},
    },
    rps::*,
    timer::*,
    user::{avatar::avatar, neko_commands::neko, snipe::snipe, math::math::math},
    utilities::configure::configure,
    utils::*,
};


// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;


pub struct PrintError(String);

impl std::fmt::Display for PrintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for PrintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for PrintError {}

#[allow(unused)]
pub type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    

    env_logger::init();


    let token = secrets::get_secret("DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT | serenity::GatewayIntents::GUILD_MEMBERS;

    let framework = poise::Framework::builder()
        .setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                poise::builtins::register_globally(_ctx, &_framework.options().commands).await?;
                {
                    events::timer::check_timer().await;
                    events::moderation::check_mutes().await;
                }

                Ok(Data {})
            })
        })
        .options(poise::FrameworkOptions {
            commands: vec![
                avatar::avatar(),
                neko(),
                configure(),
                help(),
                info(),
                ban(),
                kick(),
                unban(),
                mute(),
                unmute(),
                rock_paper_scissors(),
                timer(),
                ping(),
                math(),
                snipe(),
            ],
            on_error: |error| Box::pin(on_error(error)),
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("-".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!(
                "Logged in as {}\n--------------------------------",
                data_about_bot.user.name
            );
        }
        serenity::FullEvent::Message { new_message, .. } => {
            println!(
                "Message from {}: {}\n--------------------------------",
                new_message.author.name, new_message.content
            );
            handler::message_logging::handle_messages(new_message, _framework)
                .await?;
            handler::xp_handler::handle_xp(new_message).await?;
            handler::messages_reactions::message_reactions(new_message, &ctx).await?;
        }
        serenity::FullEvent::MessageDelete {
            channel_id,
            deleted_message_id,
            guild_id,
        } => {
            println!(
                "deleted this message: {} in guild: {}\n--------------------------------",
                deleted_message_id,
                guild_id.unwrap()
            );
            handler::message_logging::deleted_messages_handler(channel_id, deleted_message_id, &ctx)
                .await?;
        }
        serenity::FullEvent::MessageUpdate {
            old_if_available: _,
            new: _,
            event,
        } => {
            println!(
                "edited message: {:?}\nid: {}\n--------------------------------",
                event.content, event.id
            );

            let edited_msg = event.content.clone();

            match edited_msg {
                Some(content) => {
                    handler::message_logging::edited_messages_handler(&event.channel_id,&event.id, &content, ctx)
                        .await?
                }
                None => {
                    println!("edited content is None\n--------------------------------");
                }
            }
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            println!("new member: {}\n--------------------------------", new_member.user.name);
            handler::member_join::member_join(new_member, ctx).await?;
        }
        _ => {}
    }
    Ok(())
}
