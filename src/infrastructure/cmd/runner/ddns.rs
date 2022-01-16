use anyhow::Result;
use async_trait::async_trait;

use crate::{
    domain::ddns::DdnsStatus,
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpDdnsCommand,
    },
    service::Runner,
};

pub struct DdnsRunner<'a, P>
where P: Parser<Item = Vec<DdnsStatus>> + Send + Sync
{
    command: &'a OpDdnsCommand,
    parser: P,
}

impl<'a, P> DdnsRunner<'a, P>
where P: Parser<Item = Vec<DdnsStatus>> + Send + Sync
{
    pub fn new(command: &'a OpDdnsCommand, parser: P) -> Self {
        Self {
            command,
            parser,
        }
    }

    async fn statuses(&self) -> Result<Vec<DdnsStatus>> {
        let output = self.output(&self.command, &["--show-status"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

impl<P> Executor for DdnsRunner<'_, P>
where P: Parser<Item = Vec<DdnsStatus>> + Send + Sync
{
}

#[async_trait]
impl<P> Runner for DdnsRunner<'_, P>
where P: Parser<Item = Vec<DdnsStatus>> + Send + Sync
{
    type Item = Vec<DdnsStatus>;

    async fn run(&self) -> Result<Self::Item> {
        self.statuses().await
    }
}
