use async_trait::async_trait;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::{version::VersionResult, Runner},
};

#[derive(Clone)]
pub struct VersionRunner<P>
where
    P: Parser<Item = VersionResult> + Send + Sync,
{
    command: OpCommand,
    parser: P,
}

impl<P> VersionRunner<P>
where
    P: Parser<Item = VersionResult> + Send + Sync,
{
    pub fn new(command: &OpCommand, parser: P) -> Self {
        let command = command.to_owned();
        Self {
            command,
            parser,
        }
    }

    async fn version(&self) -> anyhow::Result<VersionResult> {
        let output = self.output(&self.command, &["show", "version"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

impl<P> Executor for VersionRunner<P>
where
    P: Parser<Item = VersionResult> + Send + Sync,
{
}

#[async_trait]
impl<P> Runner for VersionRunner<P>
where
    P: Parser<Item = VersionResult> + Send + Sync,
{
    type Item = VersionResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.version().await
    }
}
