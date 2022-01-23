use async_trait::async_trait;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::{version::VersionResult, Runner},
};

#[derive(Clone)]
pub struct VersionRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = VersionResult> + Send + Sync,
{
    command: OpCommand,
    executor: E,
    parser: P,
}

impl<E, P> VersionRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = VersionResult> + Send + Sync,
{
    pub fn new(command: &OpCommand, executor: E, parser: P) -> Self {
        let command = command.to_owned();
        Self {
            command,
            executor,
            parser,
        }
    }

    async fn version(&self) -> anyhow::Result<VersionResult> {
        let output = self.executor.output(&self.command, &["show", "version"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

#[async_trait]
impl<E, P> Runner for VersionRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = VersionResult> + Send + Sync,
{
    type Item = VersionResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.version().await
    }
}
