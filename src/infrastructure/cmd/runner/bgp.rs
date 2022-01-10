use anyhow::Result;

use crate::{
    domain::bgp::BGPStatus,
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::VtyshCommand,
    },
    service::Runner,
};

pub struct BGPRunner<'a, P>
where P: Parser<Item = Option<BGPStatus>>
{
    command: &'a VtyshCommand,
    parser: P,
}

impl<'a, P> BGPRunner<'a, P>
where P: Parser<Item = Option<BGPStatus>>
{
    pub fn new(command: &'a VtyshCommand, parser: P) -> Self {
        Self {
            command,
            parser,
        }
    }

    fn ipv4(&self) -> Result<Option<BGPStatus>> {
        let output = self.output(&self.command, &["-c", "show ip bgp summary"])?;
        let result = self.parser.parse(&output)?.and_then(|mut status| {
            status.neighbors.retain(|n| n.neighbor.is_ipv4());
            Some(status)
        });
        Ok(result)
    }

    fn ipv6(&self) -> Result<Option<BGPStatus>> {
        let output = self.output(&self.command, &["-c", "show bgp ipv6 summary"])?;
        let result = self.parser.parse(&output)?.and_then(|mut status| {
            status.neighbors.retain(|n| n.neighbor.is_ipv6());
            Some(status)
        });
        Ok(result)
    }
}

impl<P> Executor for BGPRunner<'_, P>
where P: Parser<Item = Option<BGPStatus>>
{
}

impl<P> Runner for BGPRunner<'_, P>
where P: Parser<Item = Option<BGPStatus>>
{
    type Item = (Option<BGPStatus>, Option<BGPStatus>);

    fn run(&self) -> Result<Self::Item> {
        Ok((self.ipv4()?, self.ipv6()?))
    }
}
