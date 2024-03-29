use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpDdnsCommand,
    },
    service::{ddns::DdnsStatusResult, Runner},
};

pub struct DdnsRunner<E, P> {
    command: OpDdnsCommand,
    executor: E,
    parser: P,
}

impl<E, P> DdnsRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Context<'static> = (), Item = DdnsStatusResult> + Send + Sync,
{
    pub fn new(command: OpDdnsCommand, executor: E, parser: P) -> Self {
        Self {
            command,
            executor,
            parser,
        }
    }

    async fn statuses(&self) -> anyhow::Result<DdnsStatusResult> {
        let output = self.executor.output(&self.command, &["--show-status"]).await?;
        let result = self.parser.parse(&output, ())?;
        Ok(result)
    }
}

impl<E, P> Runner for DdnsRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Context<'static> = (), Item = DdnsStatusResult> + Send + Sync,
{
    type Item = DdnsStatusResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.statuses().await
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use chrono::NaiveDate;
    use futures::future::ok;
    use indoc::indoc;
    use mockall::{mock, predicate::eq};
    use pretty_assertions::assert_eq;

    use crate::{
        domain::ddns::{DdnsStatus, DdnsUpdateStatus},
        infrastructure::cmd::runner::MockExecutor,
    };

    use super::*;

    mock! {
        DdnsParser {}

        impl Parser for DdnsParser {
            type Context<'a> = ();
            type Item = DdnsStatusResult;

            fn parse(&self, input: &str, context: <Self as Parser>::Context<'static>) -> anyhow::Result<<Self as Parser>::Item>;
        }
    }

    #[tokio::test]
    async fn statuses() {
        let command = OpDdnsCommand::from("/opt/vyatta/bin/sudo-users/vyatta-op-dynamic-dns.pl".to_string());
        let output = indoc! {"
            interface    : eth0
            ip address   : 192.0.2.1
            host-name    : 1.example.com
            last update  : Mon Jan  2 15:04:05 2006
            update-status: good

            interface    : eth1 [ Currently no IP address ]
            host-name    : 2.example.com
            last update  : Mon Jan  2 15:04:06 2006
            update-status: 

        "};

        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/opt/vyatta/bin/sudo-users/vyatta-op-dynamic-dns.pl", &["--show-status"]))
            .returning(|_, _| Box::pin(ok(output.to_string())));

        let mut mock_parser = MockDdnsParser::new();
        mock_parser
            .expect_parse()
            .times(1)
            .with(eq(output), eq(()))
            .returning(|_, _| Ok(vec![
                DdnsStatus {
                    interface: "eth0".to_string(),
                    ip_address: Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1))),
                    host_name: Some("1.example.com".to_string()),
                    last_update: Some(NaiveDate::from_ymd_opt(2006, 1, 2).and_then(|d| d.and_hms_opt(15, 4, 5)).unwrap()),
                    update_status: Some(DdnsUpdateStatus::Good),
                },
                DdnsStatus {
                    interface: "eth1".to_string(),
                    ip_address: None,
                    host_name: Some("2.example.com".to_string()),
                    last_update: Some(NaiveDate::from_ymd_opt(2006, 1, 2).and_then(|d| d.and_hms_opt(15, 4, 6)).unwrap()),
                    update_status: None,
                },
            ]));

        let runner = DdnsRunner::new(command, mock_executor, mock_parser);
        let actual = runner.run().await.unwrap();
        assert_eq!(actual, vec![
            DdnsStatus {
                interface: "eth0".to_string(),
                ip_address: Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1))),
                host_name: Some("1.example.com".to_string()),
                last_update: Some(NaiveDate::from_ymd_opt(2006, 1, 2).and_then(|d| d.and_hms_opt(15, 4, 5)).unwrap()),
                update_status: Some(DdnsUpdateStatus::Good),
            },
            DdnsStatus {
                interface: "eth1".to_string(),
                ip_address: None,
                host_name: Some("2.example.com".to_string()),
                last_update: Some(NaiveDate::from_ymd_opt(2006, 1, 2).and_then(|d| d.and_hms_opt(15, 4, 6)).unwrap()),
                update_status: None,
            },
        ]);
    }
}
