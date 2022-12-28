use error_stack::{Context as ErrorContext, IntoReport, Result, ResultExt};
use serenity::client::Context;
use serenity::model::guild::Member;

#[derive(Debug)]
pub struct MemberCreateEventRuntimeError;

impl std::fmt::Display for MemberCreateEventRuntimeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Bot Error: An error occured whilst running the member_add event hooks")
    }
}

impl ErrorContext for MemberCreateEventRuntimeError {}

pub async fn member_create(
    handler: &crate::Handler,
    ctx: Context,
    mut new_member: Member,
) -> Result<(), MemberCreateEventRuntimeError> {
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
