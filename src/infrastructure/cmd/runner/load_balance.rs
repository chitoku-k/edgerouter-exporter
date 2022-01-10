use anyhow::Result;

use crate::{
    domain::load_balance::LoadBalanceGroup,
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::Runner,
};

pub struct LoadBalanceRunner<'a, P>
where P: Parser<Item = Vec<LoadBalanceGroup>>
{
    command: &'a OpCommand,
    parser: P,
}

impl<'a, P> LoadBalanceRunner<'a, P>
where P: Parser<Item = Vec<LoadBalanceGroup>>
{
    pub fn new(command: &'a OpCommand, parser: P) -> Self {
        Self {
            command,
            parser,
        }
    }

    fn groups(&self) -> Result<Vec<LoadBalanceGroup>> {
        let output = self.output(&self.command, &["show", "load-balance", "watchdog"])?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

impl<P> Executor for LoadBalanceRunner<'_, P>
where P: Parser<Item = Vec<LoadBalanceGroup>>
{
}

impl<P> Runner for LoadBalanceRunner<'_, P>
where P: Parser<Item = Vec<LoadBalanceGroup>>
{
    type Item = Vec<LoadBalanceGroup>;

    fn run(&self) -> Result<Self::Item> {
        self.groups()
    }
}
