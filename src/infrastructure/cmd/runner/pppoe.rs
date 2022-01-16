use anyhow::Result;
use async_trait::async_trait;

use crate::{
    domain::pppoe::PPPoEClientSession,
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::Runner,
};

pub struct PPPoERunner<'a, P>
where P: Parser<Item = Vec<PPPoEClientSession>> + Send + Sync
{
    command: &'a OpCommand,
    parser: P,
}

impl<'a, P> PPPoERunner<'a, P>
where P: Parser<Item = Vec<PPPoEClientSession>> + Send + Sync
{
    pub fn new(command: &'a OpCommand, parser: P) -> Self {
        Self {
            command,
            parser,
        }
    }

    async fn sessions(&self) -> Result<Vec<PPPoEClientSession>> {
        let output = self.output(&self.command, &["show", "pppoe-client"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

impl<P> Executor for PPPoERunner<'_, P>
where P: Parser<Item = Vec<PPPoEClientSession>> + Send + Sync
{
}

#[async_trait]
impl<P> Runner for PPPoERunner<'_, P>
where P: Parser<Item = Vec<PPPoEClientSession>> + Send + Sync
{
    type Item = Vec<PPPoEClientSession>;

    async fn run(&self) -> Result<Self::Item> {
        self.sessions().await
    }
}
