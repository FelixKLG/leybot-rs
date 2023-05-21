use super::CommandRuntimeError;
use async_trait::async_trait;
use error_stack::{IntoReport, Report, Result, ResultExt};
use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::application::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
};

pub struct RolesCommand;

#[async_trait]
impl super::Command for RolesCommand {
    async fn execute(
        handler: &crate::Handler,
        command: &mut ApplicationCommandInteraction,
        ctx: Context,
    ) -> Result<(), CommandRuntimeError> {
        let member = match &mut command.member {
            Some(member) => member,
            None => {
                return Err(Report::new(CommandRuntimeError)
                    .attach_printable("Failed to get member from command"))
            }
        };

        let api_response = handler
            .http
            .link_client
            .get_user_by_discord(command.user.id.0)
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
    
                if purchases.workshop_dl {
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
    
    
                "Your roles have been assigned".to_string()
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
            .name("roles")
            .description("Get access to the support channels")
            .dm_permission(false)
    }
}
