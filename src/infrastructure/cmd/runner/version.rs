use async_trait::async_trait;

use crate::{
    domain::version::Version,
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::Runner,
};

pub struct VersionRunner<'a, P>
where P: Parser<Item = Version> + Send + Sync
{
    command: &'a OpCommand,
    parser: P,
}

impl<'a, P> VersionRunner<'a, P>
where P: Parser<Item = Version> + Send + Sync
{
    pub fn new(command: &'a OpCommand, parser: P) -> Self {
        Self {
            command,
            parser,
        }
    }

    async fn version(&self) -> anyhow::Result<Version> {
        let output = self.output(&self.command, &["show", "version"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

impl<P> Executor for VersionRunner<'_, P>
where P: Parser<Item = Version> + Send + Sync
{
}

#[async_trait]
impl<P> Runner for VersionRunner<'_, P>
where P: Parser<Item = Version> + Send + Sync
{
    type Item = Version;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.version().await
    }
}
