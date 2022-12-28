use error_stack::{Context as ErrorContext, IntoReport, Report, Result, ResultExt};
use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::application::{
        command::CommandOptionType,
        interaction::{
            application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
            InteractionResponseType,
        },
    },
};
#[derive(Debug)]
pub struct SteamCommandRuntimeError;

impl std::fmt::Display for SteamCommandRuntimeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Bot Error: An error occured whilst running the steam command")
    }
}

impl ErrorContext for SteamCommandRuntimeError {}

pub async fn run(
    handler: &crate::Handler,
    command: ApplicationCommandInteraction,
    ctx: Context,
) -> Result<(), SteamCommandRuntimeError> {
    let target = match command.data.options.get(0) {
        Some(target) => match target.resolved.as_ref() {
            Some(target) => target,
            None => {
                return Err(Report::new(SteamCommandRuntimeError)
                    .attach_printable("Failed to parse command target as a user"));
            }
        },
        None => {
            return Err(Report::new(SteamCommandRuntimeError)
                .attach_printable("Failed to get command target"));
        }
    };

    let CommandDataOptionValue::User(user, _member) = target else {
        return Err(Report::new(SteamCommandRuntimeError)
            .attach_printable("Failed to fetch and validate user from command target"));
    };

    let api_response = handler
        .http
        .link_client
        .get_user_by_discord(user.id.0)
        .await
        .change_context(SteamCommandRuntimeError)?;

    let interaction_reply = match api_response {
        Some(response) => format!(
            "https://steamcommunity.com/profiles/{}",
            response.data.steam_id
        ),
        None => "User is not linked.".to_string(),
    };

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.ephemeral(true).content(interaction_reply)
                })
        })
        .await
        .into_report()
        .attach_printable("Failed to send interaction response")
        .change_context(SteamCommandRuntimeError)?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("steam")
        .description("Retrieve user Steam account page.")
        .create_option(|option| {
            option
                .name("user")
                .description("User to fetch")
                .kind(CommandOptionType::User)
                .required(true)
        })
        .dm_permission(false)
}
