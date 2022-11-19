use std::str::FromStr;

use anyhow::Context;
use chrono::NaiveDateTime;
use nom::{
    branch::{alt, permutation},
    error::Error,
    bytes::complete::{tag, take_till},
    character::complete::{alphanumeric1, line_ending, newline, not_line_ending, space0, space1, u32, u64},
    combinator::{map, map_res, opt},
    multi::{many0, many1},
    sequence::{delimited, separated_pair, terminated, tuple},
    Finish, IResult,
};

use crate::{
    domain::load_balance::{
        FlowSize,
        LoadBalancePing,
        LoadBalanceStatus,
        LoadBalanceStatusInterface,
        LoadBalanceStatusStatus,
        LoadBalanceWatchdog,
        LoadBalanceWatchdogInterface,
        LoadBalanceWatchdogStatus,
    },
    infrastructure::cmd::parser::Parser,
    service::load_balance::{LoadBalanceStatusResult, LoadBalanceWatchdogResult},
};

#[derive(Clone)]
pub struct LoadBalanceStatusParser;

#[derive(Clone)]
pub struct LoadBalanceWatchdogParser;

impl Parser for LoadBalanceStatusParser {
    type Input<'a> = &'a str;
    type Item = LoadBalanceStatusResult;

    fn parse(&self, input: Self::Input<'_>) -> anyhow::Result<Self::Item> {
        parse_load_balance_status(input)
            .finish()
            .map(|(_, status)| status)
            .map_err(|e| Error::new(e.input.to_string(), e.code))
            .context("failed to parse load-balance status")
    }
}

impl Parser for LoadBalanceWatchdogParser {
    type Input<'a> = &'a str;
    type Item = LoadBalanceWatchdogResult;

    fn parse(&self, input: Self::Input<'_>) -> anyhow::Result<Self::Item> {
        parse_load_balance_watchdog(input)
            .finish()
            .map(|(_, groups)| groups)
            .map_err(|e| Error::new(e.input.to_string(), e.code))
            .context("failed to parse load-balance groups")
    }
}

fn parse_load_balance_status(input: &str) -> IResult<&str, LoadBalanceStatusResult> {
    alt((
        map(tag("load-balance is not configured"), |_| vec![]),
        many1(
            map(
                tuple((
                    delimited(
                        tuple((tag("Group"), space1)),
                        map(not_line_ending, &str::to_string),
                        opt(newline),
                    ),
                    terminated(
                        permutation((
                            delimited(
                                tuple((space1, tag("Balance Local"), space0, tag(":"), space0)),
                                map_res(not_line_ending, &str::parse),
                                newline,
                            ),
                            delimited(
                                tuple((space1, tag("Lock Local DNS"), space0, tag(":"), space0)),
                                map_res(not_line_ending, &str::parse),
                                newline,
                            ),
                            delimited(
                                tuple((space1, tag("Conntrack Flush"), space0, tag(":"), space1)),
                                map_res(not_line_ending, &str::parse),
                                newline,
                            ),
                            delimited(
                                tuple((space1, tag("Sticky Bits"), space0, tag(":"), space1, tag("0x"))),
                                map_res(not_line_ending, |s| u32::from_str_radix(s, 16)),
                                newline,
                            ),
                        )),
                        many1(line_ending),
                    ),
                    many0(
                        map(
                            tuple((
                                delimited(
                                    tuple((space1, tag("interface"), space0, tag(":"), space1)),
                                    map(not_line_ending, &str::to_string),
                                    newline,
                                ),
                                permutation((
                                    delimited(
                                        tuple((space1, tag("reachable"), space0, tag(":"), space1)),
                                        map_res(not_line_ending, &str::parse),
                                        newline,
                                    ),
                                    delimited(
                                        tuple((space1, tag("status"), space0, tag(":"), space1)),
                                        map(
                                            not_line_ending,
                                            |s| match s {
                                                "inactive" => LoadBalanceStatusStatus::Inactive,
                                                "active" => LoadBalanceStatusStatus::Active,
                                                "failover" => LoadBalanceStatusStatus::Failover,
                                                s => LoadBalanceStatusStatus::Unknown(s.to_string()),
                                            },
                                        ),
                                        newline,
                                    ),
                                    delimited(
                                        tuple((space1, tag("gateway"), space0, tag(":"), space1)),
                                        map(
                                            not_line_ending,
                                            |s: &str| {
                                                if s.is_empty() {
                                                    None
                                                } else {
                                                    Some(s.to_string())
                                                }
                                            },
                                        ),
                                        newline,
                                    ),
                                    delimited(
                                        tuple((space1, tag("route table"), space0, tag(":"), space1)),
                                        u32,
                                        newline,
                                    ),
                                    delimited(
                                        tuple((space1, tag("weight"), space0, tag(":"), space1)),
                                        map(u32, |u| u as f64 / 100.0),
                                        tuple((tag("%"), newline)),
                                    ),
                                    delimited(
                                        tuple((space1, tag("fo_priority"), space0, tag(":"), space1)),
                                        u32,
                                        newline,
                                    ),
                                    delimited(
                                        tuple((space1, tag("flows"), newline)),
                                        map(
                                            many0(
                                                tuple((
                                                    delimited(
                                                        space0,
                                                        map(take_till(|c| c == ':'), |s: &str| s.trim_end().to_string()),
                                                        tuple((tag(":"), space1)),
                                                    ),
                                                    terminated(
                                                        map_res(not_line_ending, FlowSize::from_str),
                                                        newline,
                                                    ),
                                                )),
                                            ),
                                            |v| v.into_iter().collect(),
                                        ),
                                        many1(newline),
                                    ),
                                )),
                            )),
                            |(
                                interface,
                                (
                                    reachable,
                                    status,
                                    gateway,
                                    route_table,
                                    weight,
                                    fo_priority,
                                    flows,
                                ),
                            )| {
                                LoadBalanceStatusInterface {
                                    interface,
                                    reachable,
                                    status,
                                    gateway,
                                    route_table,
                                    weight,
                                    fo_priority,
                                    flows,
                                    watchdog: None,
                                }
                            }
                        ),
                    ),
                )),
                |(
                    name,
                    (
                        balance_local,
                        lock_local_dns,
                        conntrack_flush,
                        sticky_bits,
                    ),
                    interfaces,
                )| {
                    LoadBalanceStatus {
                        name,
                        balance_local,
                        lock_local_dns,
                        conntrack_flush,
                        sticky_bits,
                        interfaces,
                    }
                },
            ),
        ),
    ))(input)
}

fn parse_load_balance_watchdog(input: &str) -> IResult<&str, LoadBalanceWatchdogResult> {
    alt((
        map(tag("load-balance is not configured"), |_| vec![]),
        many1(
            map(
                tuple((
                    delimited(
                        tuple((tag("Group"), space1)),
                        map(not_line_ending, &str::to_string),
                        opt(newline),
                    ),
                    many0(
                        terminated(
                            map(
                                tuple((
                                    delimited(
                                        space1,
                                        map(not_line_ending, &str::to_string),
                                        newline,
                                    ),
                                    permutation((
                                        delimited(
                                            tuple((space1, tag("status:"), space1)),
                                            alt((
                                                map(
                                                    delimited(
                                                        tuple((
                                                            tag("Waiting on recovery"),
                                                            space1,
                                                        )),
                                                        delimited(
                                                            tag("("),
                                                            separated_pair(u64, tag("/"), u64),
                                                            tag(")"),
                                                        ),
                                                        space0,
                                                    ),
                                                    |(m, n)| LoadBalanceWatchdogStatus::WaitOnRecovery(m, n),
                                                ),
                                                map(
                                                    terminated(alphanumeric1::<&str, _>, space0),
                                                    |s| match s {
                                                        "OK" => LoadBalanceWatchdogStatus::Ok,
                                                        "Running" => LoadBalanceWatchdogStatus::Running,
                                                        _ => LoadBalanceWatchdogStatus::Unknown(s.to_string()),
                                                    },
                                                ),
                                            )),
                                            newline,
                                        ),
                                        map(
                                            opt(delimited(
                                                space1,
                                                tag("failover-only mode"),
                                                newline,
                                            )),
                                            |o| o.is_some(),
                                        ),
                                        delimited(
                                            tuple((space1, tag("pings:"), space1)),
                                            u64,
                                            newline,
                                        ),
                                        delimited(
                                            tuple((space1, tag("fails:"), space1)),
                                            u64,
                                            newline,
                                        ),
                                        delimited(
                                            tuple((space1, tag("run fails:"), space1)),
                                            separated_pair(u64, tag("/"), u64),
                                            newline,
                                        ),
                                        delimited(
                                            tuple((space1, tag("route drops:"), space1)),
                                            u64,
                                            newline,
                                        ),
                                        delimited(
                                            tuple((space1, tag("ping gateway:"), space1)),
                                            map(
                                                separated_pair(
                                                    take_till(|c| c == ' '),
                                                    tuple((space1, tag("-"), space1)),
                                                    not_line_ending,
                                                ),
                                                |(gateway, status)| match status {
                                                    "REACHABLE" => LoadBalancePing::Reachable(gateway.to_string()),
                                                    "DOWN" => LoadBalancePing::Down(gateway.to_string()),
                                                    _ => LoadBalancePing::Unknown(status.to_string(), gateway.to_string()),
                                                },
                                            ),
                                            newline,
                                        ),
                                        opt(delimited(
                                            tuple((space1, tag("last route drop"), space0, tag(":"), space1)),
                                            map_res(not_line_ending, |s| NaiveDateTime::parse_from_str(s, "%a %b %e %H:%M:%S %Y")),
                                            newline,
                                        )),
                                        opt(delimited(
                                            tuple((space1, tag("last route recover"), space0, tag(":"), space1)),
                                            map_res(not_line_ending, |s| NaiveDateTime::parse_from_str(s, "%a %b %e %H:%M:%S %Y")),
                                            newline,
                                        )),
                                    )),
                                )),
                                |(
                                    interface,
                                    (
                                        status,
                                        failover_only_mode,
                                        pings,
                                        fails,
                                        run_fails,
                                        route_drops,
                                        ping,
                                        last_route_drop,
                                        last_route_recover,
                                    ),
                                )| {
                                    LoadBalanceWatchdogInterface {
                                        interface,
                                        status,
                                        failover_only_mode,
                                        pings,
                                        fails,
                                        run_fails,
                                        route_drops,
                                        ping,
                                        last_route_drop,
                                        last_route_recover,
                                    }
                                },
                            ),
                            many1(line_ending),
                        ),
                    ),
                )),
                |(name, interfaces)| {
                    LoadBalanceWatchdog {
                        name,
                        interfaces,
                    }
                },
            ),
        ),
    ))(input)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::NaiveDate;
    use indoc::indoc;
    use number_prefix::NumberPrefix;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn empty() {
        let parser = LoadBalanceStatusParser;
        let input = "";

        let actual = parser.parse(input);
        assert!(actual.is_err());

        let parser = LoadBalanceWatchdogParser;
        let input = "";

        let actual = parser.parse(input);
        assert!(actual.is_err());
    }

    #[test]
    fn no_config() {
        let parser = LoadBalanceStatusParser;
        let input = "load-balance is not configured";

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, vec![]);

        let parser = LoadBalanceWatchdogParser;
        let input = "load-balance is not configured";

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, vec![]);
    }

    #[test]
    fn no_interfaces() {
        let parser = LoadBalanceStatusParser;
        let input = indoc! {"
            Group FAILOVER_01
                Balance Local  : false
                Lock Local DNS : false
                Conntrack Flush: false
                Sticky Bits    : 0x00000000

        "};

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, vec![
            LoadBalanceStatus {
                name: "FAILOVER_01".to_string(),
                balance_local: false,
                lock_local_dns: false,
                conntrack_flush: false,
                sticky_bits: 0,
                interfaces: vec![],
            },
        ]);

        let parser = LoadBalanceWatchdogParser;
        let input = "Group FAILOVER_01";

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, vec![
            LoadBalanceWatchdog {
                name: "FAILOVER_01".to_string(),
                interfaces: vec![],
            },
        ]);
    }

    #[test]
    fn single_group() {
        let parser = LoadBalanceStatusParser;
        let input = indoc! {"
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

        let actual = parser.parse(input).unwrap();
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
        ]);

        let parser = LoadBalanceWatchdogParser;
        let input = indoc! {"
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

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, vec![
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
        ]);
    }

    #[test]
    fn multiple_groups() {
        let parser = LoadBalanceStatusParser;
        let input = indoc! {"
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

            Group FAILOVER_02
                Balance Local  : true
                Lock Local DNS : true
                Conntrack Flush: true
                Sticky Bits    : 0x000000ff

              interface   : eth2
              reachable   : true
              status      : failover
              gateway     : 
              route table : 3
              weight      : 0%
              fo_priority : 60
              flows
                  WAN Out   : 2000
                  WAN In    : 2100
                  Local ICMP: 1000
                  Local DNS : 0
                  Local Data: 0

              interface   : eth3
              reachable   : true
              status      : active
              gateway     : 
              route table : 4
              weight      : 50%
              fo_priority : 60
              flows
                  WAN Out   : 3000
                  WAN In    : 3100
                  Local ICMP: 1000
                  Local DNS : 0
                  Local Data: 0

              interface   : eth4
              reachable   : true
              status      : active
              gateway     : 
              route table : 5
              weight      : 50%
              fo_priority : 60
              flows
                  WAN Out   : 4000
                  WAN In    : 4100
                  Local ICMP: 1000
                  Local DNS : 0
                  Local Data: 0

        "};

        let actual = parser.parse(input).unwrap();
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
            LoadBalanceStatus {
                name: "FAILOVER_02".to_string(),
                balance_local: true,
                lock_local_dns: true,
                conntrack_flush: true,
                sticky_bits: 0x000000ff,
                interfaces: vec![
                    LoadBalanceStatusInterface {
                        interface: "eth2".to_string(),
                        reachable: true,
                        status: LoadBalanceStatusStatus::Failover,
                        gateway: None,
                        route_table: 3,
                        weight: 0.0,
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
                        interface: "eth3".to_string(),
                        reachable: true,
                        status: LoadBalanceStatusStatus::Active,
                        gateway: None,
                        route_table: 4,
                        weight: 0.5,
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
                    LoadBalanceStatusInterface {
                        interface: "eth4".to_string(),
                        reachable: true,
                        status: LoadBalanceStatusStatus::Active,
                        gateway: None,
                        route_table: 5,
                        weight: 0.5,
                        fo_priority: 60,
                        flows: {
                            let mut flows = BTreeMap::new();
                            flows.insert("WAN Out".to_string(), NumberPrefix::Standalone(4000).into());
                            flows.insert("WAN In".to_string(), NumberPrefix::Standalone(4100).into());
                            flows.insert("Local ICMP".to_string(), NumberPrefix::Standalone(1000).into());
                            flows.insert("Local DNS".to_string(), NumberPrefix::Standalone(0).into());
                            flows.insert("Local Data".to_string(), NumberPrefix::Standalone(0).into());
                            flows
                        },
                        watchdog: None,
                    },
                ],
            },
        ]);

        let parser = LoadBalanceWatchdogParser;
        let input = indoc! {"
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

            Group FAILOVER_02
              eth2
              status: OK
              pings: 1000
              fails: 0
              run fails: 0/3
              route drops: 0
              ping gateway: ping.ubnt.com - REACHABLE

              eth3
              status: OK
              pings: 1000
              fails: 0
              run fails: 0/3
              route drops: 0
              ping gateway: ping.ubnt.com - REACHABLE
              last route drop   : Mon Jan  2 15:04:05 2006

        "};

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, vec![
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
            LoadBalanceWatchdog {
                name: "FAILOVER_02".to_string(),
                interfaces: vec![
                    LoadBalanceWatchdogInterface {
                        interface: "eth2".to_string(),
                        status: LoadBalanceWatchdogStatus::Ok,
                        failover_only_mode: false,
                        pings: 1000,
                        fails: 0,
                        run_fails: (0, 3),
                        route_drops: 0,
                        ping: LoadBalancePing::Reachable("ping.ubnt.com".to_string()),
                        last_route_drop: None,
                        last_route_recover: None,
                    },
                    LoadBalanceWatchdogInterface {
                        interface: "eth3".to_string(),
                        status: LoadBalanceWatchdogStatus::Ok,
                        failover_only_mode: false,
                        pings: 1000,
                        fails: 0,
                        run_fails: (0, 3),
                        route_drops: 0,
                        ping: LoadBalancePing::Reachable("ping.ubnt.com".to_string()),
                        last_route_drop: Some(NaiveDate::from_ymd_opt(2006, 1, 2).and_then(|d| d.and_hms_opt(15, 4, 5)).unwrap()),
                        last_route_recover: None,
                    },
                ],
            },
        ]);
    }
}
