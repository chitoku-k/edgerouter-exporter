use async_trait::async_trait;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::{pppoe::PPPoEClientSessionResult, Runner},
};

#[derive(Clone)]
pub struct PPPoERunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = PPPoEClientSessionResult> + Send + Sync,
{
    command: OpCommand,
    executor: E,
    parser: P,
}

impl<E, P> PPPoERunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = PPPoEClientSessionResult> + Send + Sync,
{
    pub fn new(command: &OpCommand, executor: E, parser: P) -> Self {
        let command = command.to_owned();
        Self {
            command,
            executor,
            parser,
        }
    }

    async fn sessions(&self) -> anyhow::Result<PPPoEClientSessionResult> {
        let output = self.executor.output(&self.command, &["show", "pppoe-client"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

#[async_trait]
impl<E, P> Runner for PPPoERunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = PPPoEClientSessionResult> + Send + Sync,
{
    type Item = PPPoEClientSessionResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.sessions().await
    }
}
