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
pub struct GmodstoreCommandRuntimeError;

impl std::fmt::Display for GmodstoreCommandRuntimeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Bot Error: An error occurred whilst running the gmodstore command")
    }
}

impl ErrorContext for GmodstoreCommandRuntimeError {}

pub async fn run(
    handler: &crate::Handler,
    command: ApplicationCommandInteraction,
    ctx: Context,
) -> Result<(), GmodstoreCommandRuntimeError> {
    let target = match command.data.options.get(0) {
        Some(target) => match target.resolved.as_ref() {
            Some(target) => target,
            None => {
                return Err(Report::new(GmodstoreCommandRuntimeError)
                    .attach_printable("Failed to parse command target as a user"));
            }
        },
        None => {
            return Err(Report::new(GmodstoreCommandRuntimeError)
                .attach_printable("Failed to get command target"));
        }
    };

    let CommandDataOptionValue::User(user, _member) = target else {
        return Err(Report::new(GmodstoreCommandRuntimeError)
            .attach_printable("Failed to fetch and validate user from command target"));
    };

    let api_response = handler
        .http
        .link_client
        .get_user_by_discord(user.id.0)
        .await
        .change_context(GmodstoreCommandRuntimeError)?;

    let interaction_reply = match api_response {
        Some(response) => match response.gmod_store_id {
            Some(gms_id) => format!("https://www.gmodstore.com/users/{}", gms_id),
            None => "User does not have a registered GmodStore account.".to_string(),
        },
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
        .change_context(GmodstoreCommandRuntimeError)?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("gmodstore")
        .description("Retrieve user GmodStore account page.")
        .create_option(|option| {
            option
                .name("user")
                .description("User to fetch")
                .kind(CommandOptionType::User)
                .required(true)
        })
        .dm_permission(false)
}
