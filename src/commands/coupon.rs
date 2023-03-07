use crate::http::CouponBuilder;
use error_stack::{Context as ErrorContext, IntoReport, Result, ResultExt};
use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::application::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
};

#[derive(Debug)]
pub struct CouponCommandRuntimeError;

impl std::fmt::Display for CouponCommandRuntimeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Bot Error: An error occurred whilst running the coupon command")
    }
}

impl ErrorContext for CouponCommandRuntimeError {}

pub async fn run(
    handler: &crate::Handler,
    command: ApplicationCommandInteraction,
    ctx: Context,
) -> Result<(), CouponCommandRuntimeError> {
    let discord_user = &command.user;

    let api_user = handler
        .http
        .link_client
        .get_user_by_discord(discord_user.id.0)
        .await
        .change_context(CouponCommandRuntimeError)?;

    let user = match api_user {
        Some(user) => user,
        None => {
            respond(command, &ctx, "You are not linked").await?;
            return Ok(());
        }
    };

    let purchases = user
        .get_purchases()
        .await
        .change_context(CouponCommandRuntimeError)?;

    if purchases.lsac {
        respond(command, &ctx, "You already own LSAC!").await?;
        return Ok(());
    } else if !purchases.swift_ac {
        respond(
            command,
            &ctx,
            "You must own SwiftAC to get coupon for LSAC!",
        )
        .await?;
        return Ok(());
    }

    let coupons = handler
        .http
        .gmod_store_client
        .get_coupons_by_user(&user, "6c5e862b-3dcf-4769-aa6b-8a001937c56b")
        .await
        .change_context(CouponCommandRuntimeError)?;

    if let Some(coupons) = coupons {
        respond(
            command,
            &ctx,
            format!(
                "You already have a valid coupon code, use code `{}`",
                coupons[0].code
            )
            .as_str(),
        )
        .await?;
        return Ok(());
    }

    let coupon_code = cuid::cuid()
        .into_report()
        .change_context(CouponCommandRuntimeError)?;

    let coupon_builder = CouponBuilder::new(coupon_code, 25, 1, user.gmod_store_id.clone())
        .change_context(CouponCommandRuntimeError)?;

    let coupon = handler
        .http
        .gmod_store_client
        .create_coupon("6c5e862b-3dcf-4769-aa6b-8a001937c56b", coupon_builder)
        .await
        .change_context(CouponCommandRuntimeError)?;

    respond(
        command,
        &ctx,
        format!("Use code: `{}`, it expires in 7 days.", coupon.code).as_str(),
    )
    .await?;
    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("coupon")
        .description("Generate a coupon for LSAC.")
        .dm_permission(false)
}

async fn respond(
    command: ApplicationCommandInteraction,
    ctx: &Context,
    reply: &str,
) -> Result<(), CouponCommandRuntimeError> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.ephemeral(true).content(reply))
        })
        .await
        .into_report()
        .attach_printable("Failed to send interaction response")
        .change_context(CouponCommandRuntimeError)?;
    Ok(())
}
