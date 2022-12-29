#[macro_use]
extern crate log;

use error_stack::{Context as ErrorContext, IntoReport, Result, ResultExt};

use crate::misc::get_env;
use dotenv::dotenv;

pub use serenity::model::application::{
    command::Command,
    interaction::{Interaction, InteractionResponseType},
};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{async_trait, model::prelude::Member};

mod commands;
mod events;
mod http;
mod misc;

#[derive(Debug)]
struct DiscordBotBuildError;

impl std::fmt::Display for DiscordBotBuildError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Bot Error: An error occured while building the bot")
    }
}

impl ErrorContext for DiscordBotBuildError {}
#[derive(Debug)]
struct DiscordBotRuntimeError;

impl std::fmt::Display for DiscordBotRuntimeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Bot Error: An error occured while running the bot")
    }
}

impl ErrorContext for DiscordBotRuntimeError {}

pub struct Handler {
    pub http: crate::http::HttpClient,
}

#[derive(Debug)]
pub struct CommandRuntimeError;

impl std::fmt::Display for CommandRuntimeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Bot Error: An error occured whilst running the gmodstore command")
    }
}

impl ErrorContext for CommandRuntimeError {}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        debug!("Connection to Discord established!");
        info!(
            "Connected as: {}#{}",
            ready.user.name, ready.user.discriminator
        );

        debug!("Attempting to push slash commands...");
        let commands = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::coupon::register(command))
                .create_application_command(|command| commands::forceroles::register(command))
                .create_application_command(|command| commands::gmodstore::register(command))
                .create_application_command(|command| commands::purchases::register(command))
                .create_application_command(|command| commands::roles::register(command))
                .create_application_command(|command| commands::steam::register(command))
                .create_application_command(|command| commands::unlink::register(command))
        })
        .await;

        match commands {
            Ok(command) => {
                debug!("Pushed {} slash commands!", command.len());

                for i in command.iter() {
                    debug!("Registered command: {}", i.name.as_str());
                }
            }
            Err(e) => {
                error!("Failed to push slash commands");
                trace!("{:#?}", e);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(mut command) = interaction {
            debug!(
                "Received command interaction: {}",
                command.data.name.as_str()
            );

            if let Err(e) = match command.data.name.as_str() {
                "coupon" => commands::coupon::run(self, command, ctx)
                    .await
                    .change_context(CommandRuntimeError),
                "force-roles" => commands::forceroles::run(self, command, ctx)
                    .await
                    .change_context(CommandRuntimeError),
                "gmodstore" => commands::gmodstore::run(self, command, ctx)
                    .await
                    .change_context(CommandRuntimeError),
                "purchases" => commands::purchases::run(self, command, ctx)
                    .await
                    .change_context(CommandRuntimeError),
                "roles" => commands::roles::run(self, &mut command, ctx)
                    .await
                    .change_context(CommandRuntimeError),
                "steam" => commands::steam::run(self, command, ctx)
                    .await
                    .change_context(CommandRuntimeError),
                "unlink" => commands::unlink::run(self, command, ctx)
                    .await
                    .change_context(CommandRuntimeError),
                _ => {
                    error!("Unknown command: {}", command.data.name.as_str());
                    return;
                }
            } {
                error!("An error occured whilst running previous command");
                trace!("{:#?}", e);
            }
        }
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        if let Err(e) = events::member::member_create(self, ctx, new_member).await {
            error!("An error occured whilst running member create event");
            trace!("{:#?}", e);
        }
    }
}

async fn build() -> Result<(), DiscordBotBuildError> {
    debug!("Building HTTP client");
    let http = crate::http::HttpClient::new().change_context(DiscordBotBuildError)?;

    let handler = Handler { http };

    let discord_token = get_env("DISCORD_TOKEN")
        .attach_printable("Failed to read discord token")
        .change_context(DiscordBotBuildError)?;
    let intents = GatewayIntents::non_privileged() | GatewayIntents::GUILD_MEMBERS;

    debug!("Building Discord client");
    let mut client = Client::builder(discord_token, intents)
        .event_handler(handler)
        .await
        .into_report()
        .attach_printable("Failed to build client")
        .change_context(DiscordBotBuildError)?;

    client
        .start()
        .await
        .into_report()
        .attach_printable("Failed to start client")
        .change_context(DiscordBotBuildError)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), DiscordBotRuntimeError> {
    dotenv()
        .into_report()
        .attach_printable("Failed to load .env file")
        .attach_printable(
            "Ensure .env is present in root directory and you have permission to read & execute it",
        )
        .change_context(DiscordBotRuntimeError)?;

    pretty_env_logger::try_init_timed_custom_env("RUST_LOG")
        .into_report()
        .attach_printable("Failed to initialize logger")
        .change_context(DiscordBotRuntimeError)?;

    build().await.change_context(DiscordBotRuntimeError)?;

    Ok(())
}
