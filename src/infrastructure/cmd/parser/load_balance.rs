use anyhow::anyhow;
use chrono::NaiveDateTime;
use nom::{
    branch::{alt, permutation},
    bytes::complete::{tag, take_till},
    character::complete::{alphanumeric1, line_ending, newline, not_line_ending, space0, space1, u64},
    combinator::{map, map_res, opt},
    multi::{many0, many1},
    sequence::{delimited, separated_pair, terminated, tuple},
    Finish,
};

use crate::{
    domain::load_balance::{
        LoadBalanceGroup,
        LoadBalanceInterface,
        LoadBalancePing,
        LoadBalanceStatus,
    },
    infrastructure::cmd::parser::Parser,
    service::load_balance::LoadBalanceGroupResult,
};

#[derive(Clone)]
pub struct LoadBalanceParser;

impl Parser for LoadBalanceParser {
    type Item = LoadBalanceGroupResult;

    fn parse(&self, input: &str) -> anyhow::Result<Self::Item> {
        parse_load_balance_groups(input)
    }
}

fn parse_load_balance_groups(input: &str) -> anyhow::Result<LoadBalanceGroupResult> {
    match alt((
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
                                                    |(m, n)| LoadBalanceStatus::WaitOnRecovery(m, n),
                                                ),
                                                map(
                                                    terminated(alphanumeric1::<&str, _>, space0),
                                                    |s| match s {
                                                        "OK" => LoadBalanceStatus::Ok,
                                                        "Running" => LoadBalanceStatus::Running,
                                                        _ => LoadBalanceStatus::Unknown(s.to_string()),
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
                                    LoadBalanceInterface {
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
                    LoadBalanceGroup {
                        name,
                        interfaces,
                    }
                },
            ),
        ),
    ))(input).finish() {
        Ok((_, groups)) => Ok(groups),
        Err::<_, nom::error::Error<_>>(e) => Err(anyhow!(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn empty() {
        let parser = LoadBalanceParser;
        let input = "";

        let actual = parser.parse(input);
        assert!(actual.is_err());
    }

    #[test]
    fn no_config() {
        let parser = LoadBalanceParser;
        let input = "load-balance is not configured";

        let actual = parser.parse(input);
        assert!(actual.is_ok());

        let actual = actual.unwrap();
        assert_eq!(actual, vec![]);
    }

    #[test]
    fn no_interfaces() {
        let parser = LoadBalanceParser;
        let input = "Group FAILOVER_01";

        let actual = parser.parse(input);
        assert!(actual.is_ok());

        let actual = actual.unwrap();
        assert_eq!(actual, vec![
            LoadBalanceGroup {
                name: "FAILOVER_01".to_string(),
                interfaces: vec![],
            },
        ]);
    }

    #[test]
    fn single_group() {
        let parser = LoadBalanceParser;
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

        let actual = parser.parse(input);
        assert!(actual.is_ok());

        let actual = actual.unwrap();
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

    #[test]
    fn multiple_groups() {
        let parser = LoadBalanceParser;
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

        let actual = parser.parse(input);
        assert!(actual.is_ok());

        let actual = actual.unwrap();
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
            LoadBalanceGroup {
                name: "FAILOVER_02".to_string(),
                interfaces: vec![
                    LoadBalanceInterface {
                        interface: "eth2".to_string(),
                        status: LoadBalanceStatus::Ok,
                        failover_only_mode: false,
                        pings: 1000,
                        fails: 0,
                        run_fails: (0, 3),
                        route_drops: 0,
                        ping: LoadBalancePing::Reachable("ping.ubnt.com".to_string()),
                        last_route_drop: None,
                        last_route_recover: None,
                    },
                    LoadBalanceInterface {
                        interface: "eth3".to_string(),
                        status: LoadBalanceStatus::Ok,
                        failover_only_mode: false,
                        pings: 1000,
                        fails: 0,
                        run_fails: (0, 3),
                        route_drops: 0,
                        ping: LoadBalancePing::Reachable("ping.ubnt.com".to_string()),
                        last_route_drop: Some(NaiveDate::from_ymd(2006, 1, 2).and_hms(15, 4, 5)),
                        last_route_recover: None,
                    },
                ],
            },
        ]);
    }
}
