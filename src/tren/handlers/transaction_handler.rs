// trait to define handlers for the engine

use std::any::Any;

use crate::tren::{
    engine::{context::RunnerContext, runner::RunnerError},
    transactions::Transaction,
};

pub trait TransactionHandler {
    fn handle(
        &mut self,
        transaction: Transaction,
        context: &mut RunnerContext,
    ) -> Result<(), RunnerError>;
    // required for downcasting in tests
    fn as_any(&self) -> &dyn Any;
}
