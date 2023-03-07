#[macro_use]
extern crate log;

use error_stack::{Context as ErrorContext, IntoReport, Result, ResultExt};

use async_trait::async_trait;
use dotenv::dotenv;

pub use serenity::model::application::{
    command::Command as InteractionCommand,
    interaction::{Interaction, InteractionResponseType},
};
use serenity::{
    model::{gateway::Ready, prelude::Member},
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};

mod commands;
mod events;
mod http;
mod misc;

use crate::misc::get_env;
use commands::Command;

#[derive(Debug)]
struct DiscordBotBuildError;

impl std::fmt::Display for DiscordBotBuildError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Bot Error: An error occurred while building the bot")
    }
}

impl ErrorContext for DiscordBotBuildError {}
#[derive(Debug)]
struct DiscordBotRuntimeError;

impl std::fmt::Display for DiscordBotRuntimeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Bot Error: An error occurred while running the bot")
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
        fmt.write_str("Bot Error: An error occurred whilst running the gmodstore command")
    }
}

impl ErrorContext for CommandRuntimeError {}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connection to Discord established!");
        info!(
            "Connected as: {}#{}",
            ready.user.name, ready.user.discriminator
        );

        debug!("Attempting to push slash commands...");
        let commands = InteractionCommand::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    commands::coupon::CouponCommand::register(command)
                })
                .create_application_command(|command| {
                    commands::forceroles::ForceRolesCommand::register(command)
                })
                .create_application_command(|command| {
                    commands::gmodstore::GmodStoreCommand::register(command)
                })
                .create_application_command(|command| {
                    commands::purchases::PurchasesCommand::register(command)
                })
                .create_application_command(|command| {
                    commands::roles::RolesCommand::register(command)
                })
                .create_application_command(|command| {
                    commands::steam::SteamCommand::register(command)
                })
                .create_application_command(|command| {
                    commands::unlink::UnlinkCommand::register(command)
                })
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
                "coupon" => commands::coupon::CouponCommand::execute(self, &mut command, ctx)
                    .await
                    .change_context(CommandRuntimeError),
                "force-roles" => {
                    commands::forceroles::ForceRolesCommand::execute(self, &mut command, ctx)
                        .await
                        .change_context(CommandRuntimeError)
                }
                "gmodstore" => {
                    commands::gmodstore::GmodStoreCommand::execute(self, &mut command, ctx)
                        .await
                        .change_context(CommandRuntimeError)
                }
                "purchases" => {
                    commands::purchases::PurchasesCommand::execute(self, &mut command, ctx)
                        .await
                        .change_context(CommandRuntimeError)
                }
                "roles" => commands::roles::RolesCommand::execute(self, &mut command, ctx)
                    .await
                    .change_context(CommandRuntimeError),
                "steam" => commands::steam::SteamCommand::execute(self, &mut command, ctx)
                    .await
                    .change_context(CommandRuntimeError),
                "unlink" => commands::unlink::UnlinkCommand::execute(self, &mut command, ctx)
                    .await
                    .change_context(CommandRuntimeError),
                _ => {
                    error!("Unknown command: {}", command.data.name.as_str());
                    return;
                }
            } {
                debug!("An error occurred whilst running previous command");
                sentry::capture_error(&e.as_error());
                error!("{:#?}", e);
            }
        }
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        if let Err(e) = events::member::member_create(self, ctx, new_member).await {
            debug!("An error occurred whilst running member create event");
            error!("{:#?}", e);
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
    #[cfg(not(debug_assertions))]
    let _guard = sentry::init((
        env!("SENTRY_DSN"),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

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
