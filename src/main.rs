extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use dotenv::dotenv;
use std::env;

use serenity::async_trait;
pub use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::prelude::GuildId;
use serenity::prelude::*;

mod commands;
mod link;

pub struct Handler {
    pub api_http: reqwest::Client,
    pub gms_http: reqwest::Client
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        debug!("Connection to Discord established!");
        info!(
            "Connected as: {}#{}",
            ready.user.name, ready.user.discriminator
        );

        let guild_id = GuildId(
            env::var("DISCORD_GUILD")
                .expect("Expected DISCORD_GUILD in environment")
                .parse()
                .expect("Failed to parse: DISCORD_GUILD; is it valid?"),
        );

        debug!("Attempting to push slash commands...");
        let commands = guild_id
            .set_application_commands(&ctx.http, |commands| {
                commands
                    // .create_application_command(|command| commands::coupon::register(command))
                    // .create_application_command(|command| commands::forceroles::register(command))
                    .create_application_command(|command| commands::gmodstore::register(command))
                    // .create_application_command(|command| commands::purchases::register(command))
                    // .create_application_command(|command| commands::roles::register(command))
                    // .create_application_command(|command| commands::steam::register(command))
                    // .create_application_command(|command| commands::unlink::register(command))
            })
            .await;

        if let Err(e) = commands {
            error!("Failed to push slash commands");
            trace!("{:#?}", e);
        } else {
            debug!("Successfully pushed slash commands!");
            let commands = commands.unwrap();
            for i in 0..commands.len() {
                debug!("Registered command: {}", commands[i].name.as_str());
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            debug!(
                "Received command interaction: {:?}",
                command.data.name.as_str()
            );

            let operation = match command.data.name.as_str() {
                // "coupon" => commands::coupon::run(Self, command, ctx),
                // "force-roles" => commands::forceroles::run(Self, command, ctx),
                "gmodstore" => commands::gmodstore::run(self, command, ctx),
                // "purchases" => commands::purchases::run(Self, command, ctx),
                // "roles" => commands::roles::run(Self, command, ctx),
                // "steam" => commands::steam::run(Self, command, ctx),
                // "unlink" => commands::unlink::run(Self, command, ctx),
                _ => {
                    error!("Unknown command: {}", command.data.name.as_str());
                    return;
                }
            };

            // if let Err(e) = command
            //     .create_interaction_response(&ctx.http, |response| {
            //         response
            //             .kind(InteractionResponseType::ChannelMessageWithSource)
            //             .interaction_response_data(|message| message.content(content))
            //     })
            //     .await
            // {
            //     error!("Failed to respond to slash command!");
            //     trace!("{:#?}", e);
            // }

            if let Err(e) = operation.await {
                error!("Failed to run slash command!");
                trace!("{:#?}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::try_init_timed_custom_env("LOG_LEVEL").unwrap();

    let mut api_headers = reqwest::header::HeaderMap::new();
    api_headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(
            &format!(
                "Bearer {}",
                env::var("API_KEY").expect("Expected API_KEY in environment")
            )
            .as_str(),
        )
        .unwrap(),
    );

    let mut gms_headers = reqwest::header::HeaderMap::new();
    gms_headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(
            &format!(
                "Bearer {}",
                env::var("GMS_PAT").expect("Expected GMS_PAT in environment")
            )
            .as_str(),
        )
        .unwrap(),
    );

    let api_http_builder = reqwest::Client::builder()
        .default_headers(api_headers);
    let gms_http_builder = reqwest::Client::builder()
        .default_headers(gms_headers);
    
    let handler = Handler {
        api_http: api_http_builder.build().unwrap(),
        gms_http: gms_http_builder.build().unwrap()
    };

    let discord_token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");
    let intents = GatewayIntents::non_privileged();

    let mut client = Client::builder(discord_token, intents)
        .event_handler(handler)
        .await
        .expect("Error constructing client");

    debug!("Attempting to connect to Discord...");
    if let Err(e) = client.start().await {
        error!("Failed to connect to Discord!");
        trace!("{:#?}", e);
    }
}
