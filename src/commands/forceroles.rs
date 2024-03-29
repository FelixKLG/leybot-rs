use super::CommandRuntimeError;
use async_trait::async_trait;
use error_stack::{IntoReport, Report, Result, ResultExt};
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

pub struct ForceRolesCommand;

#[async_trait]
impl super::Command for ForceRolesCommand {
    async fn execute(
        handler: &crate::Handler,
        command: &mut ApplicationCommandInteraction,
        ctx: Context,
    ) -> Result<(), CommandRuntimeError> {
        let target = match command.data.options.get(0) {
            Some(target) => match target.resolved.as_ref() {
                Some(target) => target,
                None => {
                    return Err(Report::new(CommandRuntimeError)
                        .attach_printable("Failed to parse command target as a user"));
                }
            },
            None => {
                return Err(Report::new(CommandRuntimeError)
                    .attach_printable("Failed to get command target"));
            }
        };

        let CommandDataOptionValue::User(user, member) = target else {
            return Err(Report::new(CommandRuntimeError)
                .attach_printable("Failed to fetch and validate user from command target"));
        };

        let mut member = match member {
            Some(_) => {
                let guild_id = match command.guild_id {
                    Some(guild_id) => guild_id,
                    None => {
                        return Err(Report::new(CommandRuntimeError)
                            .attach_printable("Failed to fetch guild id from command target"));
                    }
                };

                guild_id
                    .member(&ctx.http, user.id)
                    .await
                    .into_report()
                    .attach_printable("Failed to fetch member from command target")
                    .change_context(CommandRuntimeError)?
            }
            None => {
                return Err(Report::new(CommandRuntimeError)
                    .attach_printable("Failed to fetch and validate member from command target"));
            }
        };

        let api_response = handler
            .http
            .link_client
            .get_user_by_discord(user.id.0)
            .await
            .change_context(CommandRuntimeError)?;

        let interaction_response = match api_response {
            Some(response) => {
                let purchases = response.get_purchases().await.change_context(CommandRuntimeError)?;
                if purchases.lsac {
                    member
                        .add_role(&ctx.http, 884061162482847765)
                        .await
                        .into_report()
                        .attach_printable("Failed to add LSAC role")
                        .change_context(CommandRuntimeError)?;
                }
    
                if purchases.swift_ac {
                    member
                        .add_role(&ctx.http, 884060408946757663)
                        .await
                        .into_report()
                        .attach_printable("Failed to add SwiftAC role")
                        .change_context(CommandRuntimeError)?;
                }
    
                if purchases.hit_reg {
                    member
                        .add_role(&ctx.http, 884060954294386698)
                        .await
                        .into_report()
                        .attach_printable("Failed to add HitReg role")
                        .change_context(CommandRuntimeError)?;
                }
    
                if purchases.screen_grabs {
                    member
                        .add_role(&ctx.http, 889306784551026780)
                        .await
                        .into_report()
                        .attach_printable("Failed to add ScreenGrabs role")
                        .change_context(CommandRuntimeError)?;
                }
    
                if purchases.screen_grabs {
                    member
                        .add_role(&ctx.http, 884060628128497716)
                        .await
                        .into_report()
                        .attach_printable("Failed to add WorkshopDL role")
                        .change_context(CommandRuntimeError)?;
                }
    
                if purchases.sexy_errors {
                    member
                        .add_role(&ctx.http, 884060823205609473)
                        .await
                        .into_report()
                        .attach_printable("Failed to add SexyErrors role")
                        .change_context(CommandRuntimeError)?;
                    }
    
    
                format!("Successfully added roles to {}", Mention::User(user.id))
            },
            None => "**You are not linked.** Linking your account at <https://leystryku.support/> is required before you can receive support roles.".to_string()
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
            .change_context(CommandRuntimeError)?;

        Ok(())
    }

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
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
}
