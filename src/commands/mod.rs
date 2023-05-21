use async_trait::async_trait;
use error_stack::{Context as ErrorContext, Result};
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};

pub mod coupon;
pub mod forceroles;
pub mod gmodstore;
pub mod purchases;
pub mod roles;
pub mod steam;
pub mod unlink;

#[async_trait]
pub trait Command {
    async fn execute(
        handler: &crate::Handler,
        ctx: &mut ApplicationCommandInteraction,
        interaction: Context,
    ) -> Result<(), CommandRuntimeError>;

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;
}

#[derive(Debug)]
pub struct CommandRuntimeError;

impl std::fmt::Display for CommandRuntimeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Bot Error: An error occurred whilst running a command")
    }
}

impl ErrorContext for CommandRuntimeError {}

// pub async fn run(
//     handler: &crate::Handler,
//     command: ApplicationCommandInteraction,
//     ctx: Context,
// ) -> Result<(), GmodstoreCommandRuntimeError> {}
// }
