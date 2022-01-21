use async_trait::async_trait;
use tokio::try_join;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::VtyshCommand,
    },
    service::{bgp::BGPStatusResult, Runner},
};

#[derive(Clone)]
pub struct BGPRunner<P>
where
    P: Parser<Item = BGPStatusResult> + Send + Sync,
{
    command: VtyshCommand,
    parser: P,
}

impl<P> BGPRunner<P>
where
    P: Parser<Item = BGPStatusResult> + Send + Sync,
{
    pub fn new(command: &VtyshCommand, parser: P) -> Self {
        let command = command.to_owned();
        Self {
            command,
            parser,
        }
    }

    async fn ipv4(&self) -> anyhow::Result<BGPStatusResult> {
        let output = self.output(&self.command, &["-c", "show ip bgp summary"]).await?;
        let result = self.parser.parse(&output)?.and_then(|mut status| {
            status.neighbors.retain(|n| n.neighbor.is_ipv4());
            Some(status)
        });
        Ok(result)
    }

    async fn ipv6(&self) -> anyhow::Result<BGPStatusResult> {
        let output = self.output(&self.command, &["-c", "show bgp ipv6 summary"]).await?;
        let result = self.parser.parse(&output)?.and_then(|mut status| {
            status.neighbors.retain(|n| n.neighbor.is_ipv6());
            Some(status)
        });
        Ok(result)
    }
}

impl<P> Executor for BGPRunner<P>
where
    P: Parser<Item = BGPStatusResult> + Send + Sync,
{
}

#[async_trait]
impl<P> Runner for BGPRunner<P>
where
    P: Parser<Item = BGPStatusResult> + Send + Sync,
{
    type Item = (BGPStatusResult, BGPStatusResult);

    async fn run(&self) -> anyhow::Result<Self::Item> {
        try_join!(self.ipv4(), self.ipv6())
    }
}
