use super::CommandRuntimeError;
use crate::misc::bool_to_emoji as emoji_parse;
use async_trait::async_trait;
use error_stack::{IntoReport, Report, Result, ResultExt};
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed, CreateEmbedAuthor},
    client::Context,
    model::{
        application::{
            command::CommandOptionType,
            interaction::{
                application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
                InteractionResponseType,
            },
        },
        Permissions,
    },
};

pub struct PurchasesCommand;

#[async_trait]
impl super::Command for PurchasesCommand {
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

        let CommandDataOptionValue::User(user, _member) = target else {
            return Err(Report::new(CommandRuntimeError)
                .attach_printable("Failed to fetch and validate user from command target"));
        };

        let api_response = handler
            .http
            .link_client
            .get_user_by_discord(user.id.0)
            .await
            .change_context(CommandRuntimeError)?;

        let mut message_reply = CreateEmbed::default();
        message_reply.title("User Purchases");
        message_reply.colour(serenity::utils::Colour::from(0xBF8AE0));

        let mut author = CreateEmbedAuthor::default();
        author.name(format!("{}#{}", user.name, user.discriminator));
        author.icon_url(
            user.avatar_url()
                .unwrap_or_else(|| user.default_avatar_url()),
        );

        message_reply.set_author(author);

        match api_response {
            Some(user) => {
                let purchases = user
                    .get_purchases()
                    .await
                    .change_context(CommandRuntimeError)?;
                let message_contents = format!(
                    r#"{} | Ley's Server-Side AntiCheat
                {} | SwiftAC
                {} | Ley's HitReg
                {} | Ley's Screengrabs
                {} | Ley WorkshopDL
                {} | Ley Sexy Errors"#,
                    emoji_parse(purchases.lsac),
                    emoji_parse(purchases.swift_ac),
                    emoji_parse(purchases.hit_reg),
                    emoji_parse(purchases.screen_grabs),
                    emoji_parse(purchases.workshop_dl),
                    emoji_parse(purchases.sexy_errors)
                );

                message_reply.description(message_contents);
            }
            None => {
                message_reply.title("User is not linked");
                message_reply
                    .description("The user is not linked or has no valid GmodStore account.");
            }
        };

        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.ephemeral(true).add_embed(message_reply)
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
            .name("purchases")
            .description("Retrieve user's GmodStore purchases.")
            .create_option(|option| {
                option
                    .name("user")
                    .description("User to fetch purchases for.")
                    .kind(CommandOptionType::User)
                    .required(true)
            })
            .dm_permission(false)
            .default_member_permissions(Permissions::MODERATE_MEMBERS)
    }
}
