use async_trait::async_trait;
use tokio::try_join;

use crate::{
    domain::bgp::BGPStatus,
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::VtyshCommand,
    },
    service::Runner,
};

pub struct BGPRunner<'a, P>
where P: Parser<Item = Option<BGPStatus>> + Send + Sync
{
    command: &'a VtyshCommand,
    parser: P,
}

impl<'a, P> BGPRunner<'a, P>
where P: Parser<Item = Option<BGPStatus>> + Send + Sync
{
    pub fn new(command: &'a VtyshCommand, parser: P) -> Self {
        Self {
            command,
            parser,
        }
    }

    async fn ipv4(&self) -> anyhow::Result<Option<BGPStatus>> {
        let output = self.output(&self.command, &["-c", "show ip bgp summary"]).await?;
        let result = self.parser.parse(&output)?.and_then(|mut status| {
            status.neighbors.retain(|n| n.neighbor.is_ipv4());
            Some(status)
        });
        Ok(result)
    }

    async fn ipv6(&self) -> anyhow::Result<Option<BGPStatus>> {
        let output = self.output(&self.command, &["-c", "show bgp ipv6 summary"]).await?;
        let result = self.parser.parse(&output)?.and_then(|mut status| {
            status.neighbors.retain(|n| n.neighbor.is_ipv6());
            Some(status)
        });
        Ok(result)
    }
}

impl<P> Executor for BGPRunner<'_, P>
where P: Parser<Item = Option<BGPStatus>> + Send + Sync
{
}

#[async_trait]
impl<P> Runner for BGPRunner<'_, P>
where P: Parser<Item = Option<BGPStatus>> + Send + Sync
{
    type Item = (Option<BGPStatus>, Option<BGPStatus>);

    async fn run(&self) -> anyhow::Result<Self::Item> {
        try_join!(self.ipv4(), self.ipv6())
    }
}
