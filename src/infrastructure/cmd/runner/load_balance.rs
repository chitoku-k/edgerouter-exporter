use async_trait::async_trait;

use crate::{
    domain::load_balance::LoadBalanceGroup,
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::Runner,
};

pub struct LoadBalanceRunner<'a, P>
where P: Parser<Item = Vec<LoadBalanceGroup>> + Send + Sync
{
    command: &'a OpCommand,
    parser: P,
}

impl<'a, P> LoadBalanceRunner<'a, P>
where P: Parser<Item = Vec<LoadBalanceGroup>> + Send + Sync
{
    pub fn new(command: &'a OpCommand, parser: P) -> Self {
        Self {
            command,
            parser,
        }
    }

    async fn groups(&self) -> anyhow::Result<Vec<LoadBalanceGroup>> {
        let output = self.output(&self.command, &["show", "load-balance", "watchdog"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

impl<P> Executor for LoadBalanceRunner<'_, P>
where P: Parser<Item = Vec<LoadBalanceGroup>> + Send + Sync
{
}

#[async_trait]
impl<P> Runner for LoadBalanceRunner<'_, P>
where P: Parser<Item = Vec<LoadBalanceGroup>> + Send + Sync
{
    type Item = Vec<LoadBalanceGroup>;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.groups().await
    }
}
