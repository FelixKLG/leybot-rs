use error_stack::{Context as ErrorContext, IntoReport, Result, ResultExt};
use serenity::client::Context;
use serenity::model::guild::Member;
use serenity::model::id::ChannelId;
use serenity::model::mention::Mention;

#[derive(Debug)]
pub struct MemberCreateEventRuntimeError;

impl std::fmt::Display for MemberCreateEventRuntimeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Bot Error: An error occurred whilst running the member_add event hooks")
    }
}

impl ErrorContext for MemberCreateEventRuntimeError {}

pub async fn member_create(
    handler: &crate::Handler,
    ctx: Context,
    mut new_member: Member,
) -> Result<(), MemberCreateEventRuntimeError> {
    let guild_channels = new_member
        .guild_id
        .channels(&ctx.http)
        .await
        .into_report()
        .attach_printable("Failed to fetch guild from member creation event")
        .change_context(MemberCreateEventRuntimeError)?;

    let welcome_channel_id = ChannelId(884064278112522260);

    let channel = guild_channels.get(&welcome_channel_id);

    if let Some(channel) = channel {
        channel
            .send_message(&ctx.http, |message| {
                message
                    .add_embed(|embed| {
                        embed
                            .title("Welcome")
                            .description(format!(
                                "Welcome to the support server for Leystryku's GmodStore addons.
                If you are not already verified please read {}",
                                Mention::Channel(ChannelId(884069163306479647))
                            ))
                            .field(
                                "**Please remember to read the rules**",
                                Mention::Channel(ChannelId(884050630241550376)),
                                false,
                            )
                            .colour(serenity::utils::Colour::from(0x85F2F2))
                    })
                    .content(Mention::User(new_member.user.id))
            })
            .await
            .into_report()
            .attach_printable("Failed to send message to welcome channel")
            .change_context(MemberCreateEventRuntimeError)?;
    }

    let user = handler
        .http
        .link_client
        .get_user_by_discord(new_member.user.id.0)
        .await
        .change_context(MemberCreateEventRuntimeError)?;

    match user {
        Some(_) => {
            new_member
                .add_role(&ctx.http, 884063960582721597)
                .await
                .into_report()
                .attach_printable_lazy(|| {
                    format!("Failed to add verified role for {}", new_member.user.tag())
                })
                .change_context(MemberCreateEventRuntimeError)?;

            Ok(())
        }
        None => Ok(()),
    }
}
