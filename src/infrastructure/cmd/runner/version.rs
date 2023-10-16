use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::{version::VersionResult, Runner},
};

pub struct VersionRunner<E, P> {
    command: OpCommand,
    executor: E,
    parser: P,
}

impl<E, P> VersionRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Context<'static> = (), Item = VersionResult> + Send + Sync,
{
    pub fn new(command: OpCommand, executor: E, parser: P) -> Self {
        Self {
            command,
            executor,
            parser,
        }
    }

    async fn version(&self) -> anyhow::Result<VersionResult> {
        let output = self.executor.output(&self.command, &["show", "version"]).await?;
        let result = self.parser.parse(&output, ())?;
        Ok(result)
    }
}

impl<E, P> Runner for VersionRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Context<'static> = (), Item = VersionResult> + Send + Sync,
{
    type Item = VersionResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.version().await
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use futures::future::ok;
    use indoc::indoc;
    use mockall::{mock, predicate::eq};
    use pretty_assertions::assert_eq;

    use crate::{domain::version::Version, infrastructure::cmd::runner::MockExecutor};

    use super::*;

    mock! {
        VersionParser {}

        impl Parser for VersionParser {
            type Context<'a> = ();
            type Item = VersionResult;

            fn parse(&self, input: &str, context: <Self as Parser>::Context<'static>) -> anyhow::Result<<Self as Parser>::Item>;
        }
    }

    #[tokio::test]
    async fn version() {
        let command = OpCommand::from("/opt/vyatta/bin/vyatta-op-cmd-wrapper".to_string());
        let output = indoc! {"
            Version:      v2.0.6
            Build ID:     5208541
            Build on:     01/02/06 15:04
            Copyright:    2012-2018 Ubiquiti Networks, Inc.
            HW model:     EdgeRouter X 5-Port
            HW S/N:       000000000000
            Uptime:       01:00:00 up  1:00,  1 user,  load average: 1.00, 1.00, 1.00
        "};

        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/opt/vyatta/bin/vyatta-op-cmd-wrapper", &["show", "version"]))
            .returning(|_, _| Box::pin(ok(output.to_string())));

        let mut mock_parser = MockVersionParser::new();
        mock_parser
            .expect_parse()
            .times(1)
            .with(eq(output), eq(()))
            .returning(|_, _| Ok(Version {
                version: "v2.0.6".to_string(),
                build_id: "5208541".to_string(),
                build_on: NaiveDate::from_ymd_opt(2006, 1, 2).and_then(|d| d.and_hms_opt(15, 4, 0)).unwrap(),
                copyright: "2012-2018 Ubiquiti Networks, Inc.".to_string(),
                hw_model: "EdgeRouter X 5-Port".to_string(),
                hw_serial_number: "000000000000".to_string(),
                uptime: "01:00:00 up  1:00,  1 user,  load average: 1.00, 1.00, 1.00".to_string(),
            }));

        let runner = VersionRunner::new(command, mock_executor, mock_parser);
        let actual = runner.run().await.unwrap();
        assert_eq!(actual, Version {
            version: "v2.0.6".to_string(),
            build_id: "5208541".to_string(),
            build_on: NaiveDate::from_ymd_opt(2006, 1, 2).and_then(|d| d.and_hms_opt(15, 4, 0)).unwrap(),
            copyright: "2012-2018 Ubiquiti Networks, Inc.".to_string(),
            hw_model: "EdgeRouter X 5-Port".to_string(),
            hw_serial_number: "000000000000".to_string(),
            uptime: "01:00:00 up  1:00,  1 user,  load average: 1.00, 1.00, 1.00".to_string(),
        });
    }
}
