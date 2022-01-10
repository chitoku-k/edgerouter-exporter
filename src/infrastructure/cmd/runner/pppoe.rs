use anyhow::Result;

use crate::{
    domain::pppoe::PPPoEClientSession,
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::Runner,
};

pub struct PPPoERunner<'a, P>
where P: Parser<Item = Vec<PPPoEClientSession>>
{
    command: &'a OpCommand,
    parser: P,
}

impl<'a, P> PPPoERunner<'a, P>
where P: Parser<Item = Vec<PPPoEClientSession>>
{
    pub fn new(command: &'a OpCommand, parser: P) -> Self {
        Self {
            command,
            parser,
        }
    }

    fn sessions(&self) -> Result<Vec<PPPoEClientSession>> {
        let output = self.output(&self.command, &["show", "pppoe-client"])?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

impl<P> Executor for PPPoERunner<'_, P>
where P: Parser<Item = Vec<PPPoEClientSession>>
{
}

impl<P> Runner for PPPoERunner<'_, P>
where P: Parser<Item = Vec<PPPoEClientSession>>
{
    type Item = Vec<PPPoEClientSession>;

    fn run(&self) -> Result<Self::Item> {
        self.sessions()
    }
}
