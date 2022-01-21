use async_trait::async_trait;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::{pppoe::PPPoEClientSessionResult, Runner},
};

#[derive(Clone)]
pub struct PPPoERunner<P>
where
    P: Parser<Item = PPPoEClientSessionResult> + Send + Sync,
{
    command: OpCommand,
    parser: P,
}

impl<P> PPPoERunner<P>
where
    P: Parser<Item = PPPoEClientSessionResult> + Send + Sync,
{
    pub fn new(command: &OpCommand, parser: P) -> Self {
        let command = command.to_owned();
        Self {
            command,
            parser,
        }
    }

    async fn sessions(&self) -> anyhow::Result<PPPoEClientSessionResult> {
        let output = self.output(&self.command, &["show", "pppoe-client"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

impl<P> Executor for PPPoERunner<P>
where
    P: Parser<Item = PPPoEClientSessionResult> + Send + Sync,
{
}

#[async_trait]
impl<P> Runner for PPPoERunner<P>
where
    P: Parser<Item = PPPoEClientSessionResult> + Send + Sync,
{
    type Item = PPPoEClientSessionResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.sessions().await
    }
}
