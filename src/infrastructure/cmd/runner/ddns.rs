use async_trait::async_trait;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpDdnsCommand,
    },
    service::{ddns::DdnsStatusResult, Runner},
};

#[derive(Clone)]
pub struct DdnsRunner<P>
where
    P: Parser<Item = DdnsStatusResult> + Send + Sync,
{
    command: OpDdnsCommand,
    parser: P,
}

impl<P> DdnsRunner<P>
where
    P: Parser<Item = DdnsStatusResult> + Send + Sync,
{
    pub fn new(command: &OpDdnsCommand, parser: P) -> Self {
        let command = command.to_owned();
        Self {
            command,
            parser,
        }
    }

    async fn statuses(&self) -> anyhow::Result<DdnsStatusResult> {
        let output = self.output(&self.command, &["--show-status"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

impl<P> Executor for DdnsRunner<P>
where
    P: Parser<Item = DdnsStatusResult> + Send + Sync,
{
}

#[async_trait]
impl<P> Runner for DdnsRunner<P>
where
    P: Parser<Item = DdnsStatusResult> + Send + Sync,
{
    type Item = DdnsStatusResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.statuses().await
    }
}
