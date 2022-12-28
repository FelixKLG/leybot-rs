use error_stack::{Context as ErrorContext, IntoReport, Report, Result, ResultExt};
use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::{
        application::{
            command::CommandOptionType,
            interaction::{
                application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
                InteractionResponseType,
            },
        },
        mention::Mention,
        permissions::Permissions,
    },
};

#[derive(Debug)]
pub struct UnlinkCommandRuntimeError;

impl std::fmt::Display for UnlinkCommandRuntimeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Bot Error: An error occured whilst running the unlink command")
    }
}

impl ErrorContext for UnlinkCommandRuntimeError {}

pub async fn run(
    handler: &crate::Handler,
    command: ApplicationCommandInteraction,
    ctx: Context,
) -> Result<(), UnlinkCommandRuntimeError> {
    let target = match command.data.options.get(0) {
        Some(target) => match target.resolved.as_ref() {
            Some(target) => target,
            None => {
                return Err(Report::new(UnlinkCommandRuntimeError)
                    .attach_printable("Failed to parse command target as a user"));
            }
        },
        None => {
            return Err(Report::new(UnlinkCommandRuntimeError)
                .attach_printable("Failed to get command target"));
        }
    };

    let CommandDataOptionValue::User(user, _member) = target else {
        return Err(Report::new(UnlinkCommandRuntimeError)
            .attach_printable("Failed to fetch and validate user from command target"));
    };

    let api_response = handler
        .http
        .link_client
        .delete_user_by_discord(user.id.0)
        .await
        .change_context(UnlinkCommandRuntimeError)?;

    let interaction_reply = match api_response {
        Some(_) => {
            format!("Unlinked {}", Mention::User(user.id))
        }
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
        .change_context(UnlinkCommandRuntimeError)?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("unlink")
        .description("Unlink user account")
        .create_option(|option| {
            option
                .name("user")
                .description("The user to unlink")
                .kind(CommandOptionType::User)
                .required(true)
        })
        .dm_permission(false)
        .default_member_permissions(Permissions::MODERATE_MEMBERS)
}
