use async_trait::async_trait;
use error_stack::Result;
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
pub trait Command<E> {
    async fn execute(
        handler: &crate::Handler,
        ctx: &mut ApplicationCommandInteraction,
        interaction: Context,
    ) -> Result<(), E>;

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;
}

// pub async fn run(
//     handler: &crate::Handler,
//     command: ApplicationCommandInteraction,
//     ctx: Context,
// ) -> Result<(), GmodstoreCommandRuntimeError> {}
// }
