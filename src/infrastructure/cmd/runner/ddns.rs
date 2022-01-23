use async_trait::async_trait;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpDdnsCommand,
    },
    service::{ddns::DdnsStatusResult, Runner},
};

#[derive(Clone)]
pub struct DdnsRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = DdnsStatusResult> + Send + Sync,
{
    command: OpDdnsCommand,
    executor: E,
    parser: P,
}

impl<E, P> DdnsRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = DdnsStatusResult> + Send + Sync,
{
    pub fn new(command: &OpDdnsCommand, executor: E, parser: P) -> Self {
        let command = command.to_owned();
        Self {
            command,
            executor,
            parser,
        }
    }

    async fn statuses(&self) -> anyhow::Result<DdnsStatusResult> {
        let output = self.executor.output(&self.command, &["--show-status"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

#[async_trait]
impl<E, P> Runner for DdnsRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = DdnsStatusResult> + Send + Sync,
{
    type Item = DdnsStatusResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.statuses().await
    }
}
