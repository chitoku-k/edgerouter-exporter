use async_trait::async_trait;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::{load_balance::LoadBalanceGroupResult, Runner},
};

#[derive(Clone)]
pub struct LoadBalanceRunner<P>
where
    P: Parser<Item = LoadBalanceGroupResult> + Send + Sync,
{
    command: OpCommand,
    parser: P,
}

impl<P> LoadBalanceRunner<P>
where
    P: Parser<Item = LoadBalanceGroupResult> + Send + Sync,
{
    pub fn new(command: &OpCommand, parser: P) -> Self {
        let command = command.to_owned();
        Self {
            command,
            parser,
        }
    }

    async fn groups(&self) -> anyhow::Result<LoadBalanceGroupResult> {
        let output = self.output(&self.command, &["show", "load-balance", "watchdog"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

impl<P> Executor for LoadBalanceRunner<P>
where
    P: Parser<Item = LoadBalanceGroupResult> + Send + Sync,
{
}

#[async_trait]
impl<P> Runner for LoadBalanceRunner<P>
where
    P: Parser<Item = LoadBalanceGroupResult> + Send + Sync,
{
    type Item = LoadBalanceGroupResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.groups().await
    }
}
