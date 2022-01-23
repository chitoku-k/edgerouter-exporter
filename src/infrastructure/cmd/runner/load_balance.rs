use async_trait::async_trait;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::{load_balance::LoadBalanceGroupResult, Runner},
};

#[derive(Clone)]
pub struct LoadBalanceRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = LoadBalanceGroupResult> + Send + Sync,
{
    command: OpCommand,
    executor: E,
    parser: P,
}

impl<E, P> LoadBalanceRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = LoadBalanceGroupResult> + Send + Sync,
{
    pub fn new(command: &OpCommand, executor: E, parser: P) -> Self {
        let command = command.to_owned();
        Self {
            command,
            executor,
            parser,
        }
    }

    async fn groups(&self) -> anyhow::Result<LoadBalanceGroupResult> {
        let output = self.executor.output(&self.command, &["show", "load-balance", "watchdog"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

#[async_trait]
impl<E, P> Runner for LoadBalanceRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = LoadBalanceGroupResult> + Send + Sync,
{
    type Item = LoadBalanceGroupResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.groups().await
    }
}
