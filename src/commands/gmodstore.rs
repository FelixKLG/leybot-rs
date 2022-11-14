use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::CommandDataOptionValue;
use std::io::Error;
use std::env;

pub async fn run(http: &crate::Handler, command: ApplicationCommandInteraction, ctx: Context) -> Result<(), Error> {
    let target = command.data.options.get(0);

    let target = match target {
        Some(target) => target,
        None => {
            let _ = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content("Failed to parse target")
                        })
                })
                .await;
                error!("Failed to parse target!");
            return Ok(());
        }
    };

    let target = match target.resolved.as_ref() {
        Some(target) => target,
        None => {
            let _ = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content("Failed to parse target")
                        })
                })
                .await;
                error!("Failed to parse target!");
            return Ok(());
        }
    };

    let CommandDataOptionValue::User(user, _member) = target else {
        error!("Failed to parse user option");
        return Err(Error::new(std::io::ErrorKind::Other, "Failed to parse user option"));
    };

    let api_user = http.api_http
        .get(format!("{}/api/users/discord/{}", env::var("API_ENDPOINT").unwrap(), user.id))
        .send()
        .await;

    let http_response = match api_user {
        Ok(http_response) => http_response,
        Err(e) => {

            let _ = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content("User not found")
                        })
                })
                .await;
            error!("Failed to get user from API!");
            trace!("{:#?}", e);
            return Ok(());
        }
    };

    let http_response = match http_response.error_for_status() {
        Ok(http_response) => http_response.json::<crate::link::ApiUserResponse>().await,
        Err(e) => {
            if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                let _ = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content("User not found")
                        })
                })
                .await;
                return Ok(());
            }
            
            let _ = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content("An error occured!")
                        })
                })
                .await;
            error!("Failed to get user from API");
            trace!("{:#?}", e);
            return Ok(());
        }
    };


    let _ = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message
                    .ephemeral(true)
                    .content(format!("https://www.gmodstore.com/users/{}", http_response.unwrap().data.gmodStoreId.unwrap()))
                )
        })
        .await;
    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    debug!("Building command: gmodstore");
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
}
