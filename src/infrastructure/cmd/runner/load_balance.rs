use async_trait::async_trait;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::{
        load_balance::{LoadBalanceStatusResult, LoadBalanceWatchdogResult},
        Runner,
    },
};

#[derive(Clone)]
pub struct LoadBalanceRunner<E, StatusParser, WatchdogParser> {
    command: OpCommand,
    executor: E,
    status_parser: StatusParser,
    watchdog_parser: WatchdogParser,
}

impl<E, StatusParser, WatchdogParser> LoadBalanceRunner<E, StatusParser, WatchdogParser>
where
    E: Executor + Send + Sync,
    for<'a> StatusParser: Parser<Input<'a> = &'a str, Item = LoadBalanceStatusResult> + Send + Sync,
    for<'a> WatchdogParser: Parser<Input<'a> = &'a str, Item = LoadBalanceWatchdogResult> + Send + Sync,
{
    pub fn new(command: OpCommand, executor: E, status_parser: StatusParser, watchdog_parser: WatchdogParser) -> Self {
        Self {
            command,
            executor,
            status_parser,
            watchdog_parser,
        }
    }

    async fn status(&self) -> anyhow::Result<LoadBalanceStatusResult> {
        let output = self.executor.output(&self.command, &["show", "load-balance", "status"]).await?;
        let result = self.status_parser.parse(&output)?;
        Ok(result)
    }

    async fn watchdog(&self) -> anyhow::Result<LoadBalanceWatchdogResult> {
        let output = self.executor.output(&self.command, &["show", "load-balance", "watchdog"]).await?;
        let result = self.watchdog_parser.parse(&output)?;
        Ok(result)
    }
}

#[async_trait]
impl<E, StatusParser, WatchdogParser> Runner for LoadBalanceRunner<E, StatusParser, WatchdogParser>
where
    E: Executor + Send + Sync,
    for<'a> StatusParser: Parser<Input<'a> = &'a str, Item = LoadBalanceStatusResult> + Send + Sync,
    for<'a> WatchdogParser: Parser<Input<'a> = &'a str, Item = LoadBalanceWatchdogResult> + Send + Sync,
{
    type Item = LoadBalanceStatusResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        let mut statuses = self.status().await?;
        let watchdogs = self.watchdog().await?;

        for status in &mut statuses {
            for interface in &mut status.interfaces {
                interface.watchdog = watchdogs
                    .iter()
                    .find(|w| w.name == status.name)
                    .map(|w| w.interfaces.as_slice())
                    .unwrap_or_default()
                    .iter()
                    .find(|i| i.interface == interface.interface)
                    .cloned();
            }
        }

        Ok(statuses)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::NaiveDate;
    use indoc::indoc;
    use mockall::{mock, predicate::eq};
    use number_prefix::NumberPrefix;
    use pretty_assertions::assert_eq;

    use crate::{
        domain::load_balance::{
            LoadBalancePing,
            LoadBalanceStatus,
            LoadBalanceStatusInterface,
            LoadBalanceStatusStatus,
            LoadBalanceWatchdog,
            LoadBalanceWatchdogInterface,
            LoadBalanceWatchdogStatus,
        },
        infrastructure::cmd::{parser::Parser, runner::MockExecutor},
    };

    use super::*;

    mock! {
        LoadBalanceStatusParser {}

        impl Parser for LoadBalanceStatusParser {
            type Input<'a> = &'a str;
            type Item = LoadBalanceStatusResult;

            fn parse(&self, input: &str) -> anyhow::Result<<Self as Parser>::Item>;
        }
    }

    mock! {
        LoadBalanceWatchdogParser {}

        impl Parser for LoadBalanceWatchdogParser {
            type Input<'a> = &'a str;
            type Item = LoadBalanceWatchdogResult;

            fn parse(&self, input: &str) -> anyhow::Result<<Self as Parser>::Item>;
        }
    }

    #[tokio::test]
    async fn groups() {
        let command = OpCommand::from("/opt/vyatta/bin/vyatta-op-cmd-wrapper".to_string());
        let status_output = indoc! {"
            Group FAILOVER_01
                Balance Local  : false
                Lock Local DNS : false
                Conntrack Flush: false
                Sticky Bits    : 0x00000000

              interface   : eth0
              reachable   : true
              status      : active
              gateway     : 
              route table : 1
              weight      : 100%
              fo_priority : 60
              flows
                  WAN Out   : 2000
                  WAN In    : 2100
                  Local ICMP: 1000
                  Local DNS : 0
                  Local Data: 0

              interface   : eth1
              reachable   : false
              status      : inactive
              gateway     : 
              route table : 2
              weight      : 0%
              fo_priority : 60
              flows
                  WAN Out   : 3000
                  WAN In    : 3100
                  Local ICMP: 1000
                  Local DNS : 0
                  Local Data: 0

        "};

        let watchdog_output = indoc! {"
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
            .withf(|command, args| (command, args) == ("/opt/vyatta/bin/vyatta-op-cmd-wrapper", &["show", "load-balance", "status"]))
            .returning(|_, _| Ok(status_output.to_string()));

        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/opt/vyatta/bin/vyatta-op-cmd-wrapper", &["show", "load-balance", "watchdog"]))
            .returning(|_, _| Ok(watchdog_output.to_string()));

        let mut mock_status_parser = MockLoadBalanceStatusParser::new();
        mock_status_parser
            .expect_parse()
            .times(1)
            .with(eq(status_output))
            .returning(|_| Ok(vec![
                LoadBalanceStatus {
                    name: "FAILOVER_01".to_string(),
                    balance_local: false,
                    lock_local_dns: false,
                    conntrack_flush: false,
                    sticky_bits: 0,
                    interfaces: vec![
                        LoadBalanceStatusInterface {
                            interface: "eth0".to_string(),
                            reachable: true,
                            status: LoadBalanceStatusStatus::Active,
                            gateway: None,
                            route_table: 1,
                            weight: 1.0,
                            fo_priority: 60,
                            flows: {
                                let mut flows = BTreeMap::new();
                                flows.insert("WAN Out".to_string(), NumberPrefix::Standalone(2000).into());
                                flows.insert("WAN In".to_string(), NumberPrefix::Standalone(2100).into());
                                flows.insert("Local ICMP".to_string(), NumberPrefix::Standalone(1000).into());
                                flows.insert("Local DNS".to_string(), NumberPrefix::Standalone(0).into());
                                flows.insert("Local Data".to_string(), NumberPrefix::Standalone(0).into());
                                flows
                            },
                            watchdog: None,
                        },
                        LoadBalanceStatusInterface {
                            interface: "eth1".to_string(),
                            reachable: false,
                            status: LoadBalanceStatusStatus::Inactive,
                            gateway: None,
                            route_table: 2,
                            weight: 0.0,
                            fo_priority: 60,
                            flows: {
                                let mut flows = BTreeMap::new();
                                flows.insert("WAN Out".to_string(), NumberPrefix::Standalone(3000).into());
                                flows.insert("WAN In".to_string(), NumberPrefix::Standalone(3100).into());
                                flows.insert("Local ICMP".to_string(), NumberPrefix::Standalone(1000).into());
                                flows.insert("Local DNS".to_string(), NumberPrefix::Standalone(0).into());
                                flows.insert("Local Data".to_string(), NumberPrefix::Standalone(0).into());
                                flows
                            },
                            watchdog: None,
                        },
                    ],
                },
            ]));

        let mut mock_watchdog_parser = MockLoadBalanceWatchdogParser::new();
        mock_watchdog_parser
            .expect_parse()
            .times(1)
            .with(eq(watchdog_output))
            .returning(|_| Ok(vec![
                LoadBalanceWatchdog {
                    name: "FAILOVER_01".to_string(),
                    interfaces: vec![
                        LoadBalanceWatchdogInterface {
                            interface: "eth0".to_string(),
                            status: LoadBalanceWatchdogStatus::Ok,
                            failover_only_mode: true,
                            pings: 1000,
                            fails: 1,
                            run_fails: (0, 3),
                            route_drops: 0,
                            ping: LoadBalancePing::Reachable("ping.ubnt.com".to_string()),
                            last_route_drop: None,
                            last_route_recover: None,
                        },
                        LoadBalanceWatchdogInterface {
                            interface: "eth1".to_string(),
                            status: LoadBalanceWatchdogStatus::WaitOnRecovery(0, 3),
                            failover_only_mode: false,
                            pings: 1000,
                            fails: 10,
                            run_fails: (3, 3),
                            route_drops: 1,
                            ping: LoadBalancePing::Down("ping.ubnt.com".to_string()),
                            last_route_drop: Some(NaiveDate::from_ymd_opt(2006, 1, 2).and_then(|d| d.and_hms_opt(15, 4, 5)).unwrap()),
                            last_route_recover: Some(NaiveDate::from_ymd_opt(2006, 1, 2).and_then(|d| d.and_hms_opt(15, 4, 0)).unwrap()),
                        },
                    ],
                },
            ]));

        let runner = LoadBalanceRunner::new(command, mock_executor, mock_status_parser, mock_watchdog_parser);
        let actual = runner.run().await.unwrap();
        assert_eq!(actual, vec![
            LoadBalanceStatus {
                name: "FAILOVER_01".to_string(),
                balance_local: false,
                lock_local_dns: false,
                conntrack_flush: false,
                sticky_bits: 0,
                interfaces: vec![
                    LoadBalanceStatusInterface {
                        interface: "eth0".to_string(),
                        reachable: true,
                        status: LoadBalanceStatusStatus::Active,
                        gateway: None,
                        route_table: 1,
                        weight: 1.0,
                        fo_priority: 60,
                        flows: {
                            let mut flows = BTreeMap::new();
                            flows.insert("WAN Out".to_string(), NumberPrefix::Standalone(2000).into());
                            flows.insert("WAN In".to_string(), NumberPrefix::Standalone(2100).into());
                            flows.insert("Local ICMP".to_string(), NumberPrefix::Standalone(1000).into());
                            flows.insert("Local DNS".to_string(), NumberPrefix::Standalone(0).into());
                            flows.insert("Local Data".to_string(), NumberPrefix::Standalone(0).into());
                            flows
                        },
                        watchdog: Some(LoadBalanceWatchdogInterface {
                            interface: "eth0".to_string(),
                            status: LoadBalanceWatchdogStatus::Ok,
                            failover_only_mode: true,
                            pings: 1000,
                            fails: 1,
                            run_fails: (0, 3),
                            route_drops: 0,
                            ping: LoadBalancePing::Reachable("ping.ubnt.com".to_string()),
                            last_route_drop: None,
                            last_route_recover: None,
                        }),
                    },
                    LoadBalanceStatusInterface {
                        interface: "eth1".to_string(),
                        reachable: false,
                        status: LoadBalanceStatusStatus::Inactive,
                        gateway: None,
                        route_table: 2,
                        weight: 0.0,
                        fo_priority: 60,
                        flows: {
                            let mut flows = BTreeMap::new();
                            flows.insert("WAN Out".to_string(), NumberPrefix::Standalone(3000).into());
                            flows.insert("WAN In".to_string(), NumberPrefix::Standalone(3100).into());
                            flows.insert("Local ICMP".to_string(), NumberPrefix::Standalone(1000).into());
                            flows.insert("Local DNS".to_string(), NumberPrefix::Standalone(0).into());
                            flows.insert("Local Data".to_string(), NumberPrefix::Standalone(0).into());
                            flows
                        },
                        watchdog: Some(LoadBalanceWatchdogInterface {
                            interface: "eth1".to_string(),
                            status: LoadBalanceWatchdogStatus::WaitOnRecovery(0, 3),
                            failover_only_mode: false,
                            pings: 1000,
                            fails: 10,
                            run_fails: (3, 3),
                            route_drops: 1,
                            ping: LoadBalancePing::Down("ping.ubnt.com".to_string()),
                            last_route_drop: Some(NaiveDate::from_ymd_opt(2006, 1, 2).and_then(|d| d.and_hms_opt(15, 4, 5)).unwrap()),
                            last_route_recover: Some(NaiveDate::from_ymd_opt(2006, 1, 2).and_then(|d| d.and_hms_opt(15, 4, 0)).unwrap()),
                        }),
                    },
                ],
            },
        ]);
    }
}
