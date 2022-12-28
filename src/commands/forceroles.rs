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
        permissions::Permissions,
    },
};

#[derive(Debug)]
pub struct ForceRolesCommandRuntimeError;

impl std::fmt::Display for ForceRolesCommandRuntimeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Bot Error: An error occured whilst running the force-roles command")
    }
}

impl ErrorContext for ForceRolesCommandRuntimeError {}

pub async fn run(
    handler: &crate::Handler,
    command: ApplicationCommandInteraction,
    ctx: Context,
) -> Result<(), ForceRolesCommandRuntimeError> {
    let target = match command.data.options.get(0) {
        Some(target) => match target.resolved.as_ref() {
            Some(target) => target,
            None => {
                return Err(Report::new(ForceRolesCommandRuntimeError)
                    .attach_printable("Failed to parse command target as a user"));
            }
        },
        None => {
            return Err(Report::new(ForceRolesCommandRuntimeError)
                .attach_printable("Failed to get command target"));
        }
    };

    let CommandDataOptionValue::User(user, member) = target else {
        return Err(Report::new(ForceRolesCommandRuntimeError)
            .attach_printable("Failed to fetch and validate user from command target"));
    };

    let mut member = match member {
        Some(member) => {
            let guild_id = match member.guild_id {
                Some(guild_id) => guild_id,
                None => {
                    return Err(Report::new(ForceRolesCommandRuntimeError)
                        .attach_printable("Failed to fetch guild id from command target"));
                }
            };

            guild_id
                .member(&ctx.http, user.id)
                .await
                .into_report()
                .attach_printable("Failed to fetch member from command target")
                .change_context(ForceRolesCommandRuntimeError)?
        }
        None => {
            return Err(Report::new(ForceRolesCommandRuntimeError)
                .attach_printable("Failed to fetch and validate member from command target"));
        }
    };

    let api_response = handler
        .http
        .link_client
        .get_purchases_by_discord(user.id.0)
        .await
        .change_context(ForceRolesCommandRuntimeError)?;

    let interaction_response = match api_response {
        Some(response) => {
            if response.data.lsac {
                member
                    .add_role(&ctx.http, 884061162482847765)
                    .await
                    .into_report()
                    .attach_printable("Failed to add LSAC role")
                    .change_context(ForceRolesCommandRuntimeError)?;
            }

            if response.data.swift_ac {
                member
                    .add_role(&ctx.http, 884060408946757663)
                    .await
                    .into_report()
                    .attach_printable("Failed to add SwiftAC role")
                    .change_context(ForceRolesCommandRuntimeError)?;
            }

            if response.data.hit_reg {
                member
                    .add_role(&ctx.http, 884060954294386698)
                    .await
                    .into_report()
                    .attach_printable("Failed to add HitReg role")
                    .change_context(ForceRolesCommandRuntimeError)?;
            }

            if response.data.screen_grabs {
                member
                    .add_role(&ctx.http, 889306784551026780)
                    .await
                    .into_report()
                    .attach_printable("Failed to add ScreenGrabs role")
                    .change_context(ForceRolesCommandRuntimeError)?;
            }

            if response.data.screen_grabs {
                member
                    .add_role(&ctx.http, 884060628128497716)
                    .await
                    .into_report()
                    .attach_printable("Failed to add WorkshopDL role")
                    .change_context(ForceRolesCommandRuntimeError)?;
            }

            if response.data.sexy_errors {
                member
                    .add_role(&ctx.http, 884060823205609473)
                    .await
                    .into_report()
                    .attach_printable("Failed to add SexyErrors role")
                    .change_context(ForceRolesCommandRuntimeError)?;
                }


            "Your roles have been assigned".to_string()
        },
        None => "**You are not linked.** Linking your account at <https://leystryku.support/> is required before you can recieve support roles.".to_string()
    };

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.ephemeral(true).content(interaction_response)
                })
        })
        .await
        .into_report()
        .attach_printable("Failed to send interaction response")
        .change_context(ForceRolesCommandRuntimeError)?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("force-roles")
        .description("Forcefully assign roles to a user")
        .create_option(|option| {
            option
                .name("member")
                .description("User to force roles upon.")
                .kind(CommandOptionType::User)
                .required(true)
        })
        .dm_permission(false)
        .default_member_permissions(Permissions::MODERATE_MEMBERS)
}
