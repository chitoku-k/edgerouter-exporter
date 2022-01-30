use async_trait::async_trait;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::{load_balance::LoadBalanceGroupResult, Runner},
};

#[derive(Clone)]
pub struct LoadBalanceRunner<E, P> {
    command: OpCommand,
    executor: E,
    parser: P,
}

impl<E, P> LoadBalanceRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Input = String, Item = LoadBalanceGroupResult> + Send + Sync,
{
    pub fn new(command: &OpCommand, executor: E, parser: P) -> Self {
        let command = command.to_owned();
        Self {
            command,
            executor,
            parser,
        }
    }

    async fn groups(&self) -> anyhow::Result<LoadBalanceGroupResult> {
        let output = self.executor.output(&self.command, &["show", "load-balance", "watchdog"]).await?;
        let result = self.parser.parse(output)?;
        Ok(result)
    }
}

#[async_trait]
impl<E, P> Runner for LoadBalanceRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Input = String, Item = LoadBalanceGroupResult> + Send + Sync,
{
    type Item = LoadBalanceGroupResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.groups().await
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use indoc::indoc;
    use mockall::{mock, predicate::eq};
    use pretty_assertions::assert_eq;

    use crate::{
        domain::load_balance::{LoadBalanceGroup, LoadBalanceInterface, LoadBalancePing, LoadBalanceStatus},
        infrastructure::cmd::{parser::Parser, runner::MockExecutor},
    };

    use super::*;

    mock! {
        LoadBalanceParser {}

        impl Parser for LoadBalanceParser {
            type Input = String;
            type Item = LoadBalanceGroupResult;

            fn parse(&self, input: String) -> anyhow::Result<<Self as Parser>::Item>;
        }
    }

    #[tokio::test]
    async fn groups() {
        let command = OpCommand::from("/opt/vyatta/bin/vyatta-op-cmd-wrapper".to_string());
        let output = indoc! {"
            Group FAILOVER_01
              eth0
              status: OK
              failover-only mode
              pings: 1000
              fails: 1
              run fails: 0/3
              route drops: 0
              ping gateway: ping.ubnt.com - REACHABLE

              eth1
              status: Waiting on recovery (0/3)
              pings: 1000
              fails: 10
              run fails: 3/3
              route drops: 1
              ping gateway: ping.ubnt.com - DOWN
              last route drop   : Mon Jan  2 15:04:05 2006
              last route recover: Mon Jan  2 15:04:00 2006

        "};

        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/opt/vyatta/bin/vyatta-op-cmd-wrapper", &["show", "load-balance", "watchdog"]))
            .returning(|_, _| Ok(output.to_string()));

        let mut mock_parser = MockLoadBalanceParser::new();
        mock_parser
            .expect_parse()
            .times(1)
            .with(eq(output.to_string()))
            .returning(|_| Ok(vec![
                LoadBalanceGroup {
                    name: "FAILOVER_01".to_string(),
                    interfaces: vec![
                        LoadBalanceInterface {
                            interface: "eth0".to_string(),
                            status: LoadBalanceStatus::Ok,
                            failover_only_mode: true,
                            pings: 1000,
                            fails: 1,
                            run_fails: (0, 3),
                            route_drops: 0,
                            ping: LoadBalancePing::Reachable("ping.ubnt.com".to_string()),
                            last_route_drop: None,
                            last_route_recover: None,
                        },
                        LoadBalanceInterface {
                            interface: "eth1".to_string(),
                            status: LoadBalanceStatus::WaitOnRecovery(0, 3),
                            failover_only_mode: false,
                            pings: 1000,
                            fails: 10,
                            run_fails: (3, 3),
                            route_drops: 1,
                            ping: LoadBalancePing::Down("ping.ubnt.com".to_string()),
                            last_route_drop: Some(NaiveDate::from_ymd(2006, 1, 2).and_hms(15, 4, 5)),
                            last_route_recover: Some(NaiveDate::from_ymd(2006, 1, 2).and_hms(15, 4, 0)),
                        },
                    ],
                },
            ]));

        let runner = LoadBalanceRunner::new(&command, mock_executor, mock_parser);
        let actual = runner.run().await.unwrap();
        assert_eq!(actual, vec![
            LoadBalanceGroup {
                name: "FAILOVER_01".to_string(),
                interfaces: vec![
                    LoadBalanceInterface {
                        interface: "eth0".to_string(),
                        status: LoadBalanceStatus::Ok,
                        failover_only_mode: true,
                        pings: 1000,
                        fails: 1,
                        run_fails: (0, 3),
                        route_drops: 0,
                        ping: LoadBalancePing::Reachable("ping.ubnt.com".to_string()),
                        last_route_drop: None,
                        last_route_recover: None,
                    },
                    LoadBalanceInterface {
                        interface: "eth1".to_string(),
                        status: LoadBalanceStatus::WaitOnRecovery(0, 3),
                        failover_only_mode: false,
                        pings: 1000,
                        fails: 10,
                        run_fails: (3, 3),
                        route_drops: 1,
                        ping: LoadBalancePing::Down("ping.ubnt.com".to_string()),
                        last_route_drop: Some(NaiveDate::from_ymd(2006, 1, 2).and_hms(15, 4, 5)),
                        last_route_recover: Some(NaiveDate::from_ymd(2006, 1, 2).and_hms(15, 4, 0)),
                    },
                ],
            },
        ]);
    }
}
