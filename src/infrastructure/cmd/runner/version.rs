use anyhow::Result;

use crate::{
    domain::version::Version,
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::Runner,
};

pub struct VersionRunner<'a, P>
where P: Parser<Item = Version>
{
    command: &'a OpCommand,
    parser: P,
}

impl<'a, P> VersionRunner<'a, P>
where P: Parser<Item = Version>
{
    pub fn new(command: &'a OpCommand, parser: P) -> Self {
        Self {
            command,
            parser,
        }
    }

    fn version(&self) -> Result<Version> {
        let output = self.output(&self.command, &["show", "version"])?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

impl<P> Executor for VersionRunner<'_, P>
where P: Parser<Item = Version>
{
}

impl<P> Runner for VersionRunner<'_, P>
where P: Parser<Item = Version>
{
    type Item = Version;

    fn run(&self) -> Result<Self::Item> {
        self.version()
    }
}