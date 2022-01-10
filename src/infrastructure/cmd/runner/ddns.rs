use anyhow::Result;

use crate::{
    domain::ddns::DdnsStatus,
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpDdnsCommand,
    },
    service::Runner,
};

pub struct DdnsRunner<'a, P>
where P: Parser<Item = Vec<DdnsStatus>>
{
    command: &'a OpDdnsCommand,
    parser: P,
}

impl<'a, P> DdnsRunner<'a, P>
where P: Parser<Item = Vec<DdnsStatus>>
{
    pub fn new(command: &'a OpDdnsCommand, parser: P) -> Self {
        Self {
            command,
            parser,
        }
    }

    fn statuses(&self) -> Result<Vec<DdnsStatus>> {
        let output = self.output(&self.command, &["--show-status"])?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

impl<P> Executor for DdnsRunner<'_, P>
where P: Parser<Item = Vec<DdnsStatus>>
{
}

impl<P> Runner for DdnsRunner<'_, P>
where P: Parser<Item = Vec<DdnsStatus>>
{
    type Item = Vec<DdnsStatus>;

    fn run(&self) -> Result<Self::Item> {
        self.statuses()
    }
}
