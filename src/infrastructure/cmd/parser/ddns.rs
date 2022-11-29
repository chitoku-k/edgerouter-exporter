use anyhow::Context;
use chrono::NaiveDateTime;
use nom::{
    branch::{alt, permutation},
    bytes::complete::{tag, take_till},
    character::complete::{alphanumeric1, line_ending, multispace0, newline, not_line_ending, space0, space1},
    combinator::{map, map_res, opt},
    error::Error,
    multi::many1,
    sequence::{delimited, terminated, tuple},
    Finish, IResult,
};

use crate::{
    domain::ddns::{DdnsStatus, DdnsUpdateStatus},
    infrastructure::cmd::parser::Parser,
    service::ddns::DdnsStatusResult,
};

pub struct DdnsParser;

impl Parser for DdnsParser {
    type Input<'a> = &'a str;
    type Item = DdnsStatusResult;

    fn parse(&self, input: Self::Input<'_>) -> anyhow::Result<Self::Item> {
        parse_ddns_status(input)
            .finish()
            .map(|(_, status)| status)
            .map_err(|e| Error::new(e.input.to_string(), e.code))
            .context("failed to parse DDNS status")
    }
}

fn parse_ddns_status(input: &str) -> IResult<&str, DdnsStatusResult> {
    alt((
        terminated(
            map(tag("Dynamic DNS not configured"), |_| vec![]),
            multispace0,
        ),
        many1(
            terminated(
                map(
                    permutation((
                        delimited(
                            tuple((tag("interface"), space0, tag(":"), space1)),
                            map(take_till(|c| c == ' ' || c == '\n'), &str::to_string),
                            tuple((not_line_ending, newline)),
                        ),
                        opt(delimited(
                            tuple((tag("ip address"), space0, tag(":"), space1)),
                            map_res(not_line_ending, &str::parse),
                            newline,
                        )),
                        opt(delimited(
                            tuple((tag("host-name"), space0, tag(":"), space1)),
                            map(not_line_ending, &str::to_string),
                            newline,
                        )),
                        opt(delimited(
                            tuple((tag("last update"), space0, tag(":"), space1)),
                            map_res(not_line_ending, |s| NaiveDateTime::parse_from_str(s, "%a %b %e %H:%M:%S %Y")),
                            newline,
                        )),
                        map(
                            opt(delimited(
                                tuple((tag("update-status"), space0, tag(":"), space1)),
                                opt(
                                    map(alphanumeric1::<&str, _>, |s| {
                                        match s.to_ascii_lowercase().as_str() {
                                            "abuse" => DdnsUpdateStatus::Abuse,
                                            "badagent" => DdnsUpdateStatus::BadAgent,
                                            "badauth" => DdnsUpdateStatus::BadAuth,
                                            "badsys" => DdnsUpdateStatus::BadSystemParameter,
                                            "blocked" => DdnsUpdateStatus::Blocked,
                                            "dnserr" => DdnsUpdateStatus::DNSError,
                                            "failed" => DdnsUpdateStatus::Failed,
                                            "good" => DdnsUpdateStatus::Good,
                                            "illegal" => DdnsUpdateStatus::Illegal,
                                            "noaccess" => DdnsUpdateStatus::NoAccess,
                                            "nochg" | "nochange" => DdnsUpdateStatus::NoChange,
                                            "noconnect" => DdnsUpdateStatus::NoConnect,
                                            "noerror" => DdnsUpdateStatus::NoError,
                                            "nofqdn" | "notfqdn" => DdnsUpdateStatus::NoFQDN,
                                            "nohost" => DdnsUpdateStatus::NoHost,
                                            "noservice" => DdnsUpdateStatus::NoService,
                                            "!active" => DdnsUpdateStatus::NotActive,
                                            "!donator" => DdnsUpdateStatus::NotDonator,
                                            "notdyn" => DdnsUpdateStatus::NotDynamicHost,
                                            "!yours" => DdnsUpdateStatus::NotYours,
                                            "numhost" => DdnsUpdateStatus::NumHost,
                                            "toosoon" => DdnsUpdateStatus::TooSoon,
                                            "unauth" => DdnsUpdateStatus::Unauthenticated,
                                            _ => DdnsUpdateStatus::Unknown(s.to_string()),
                                        }
                                    }),
                                ),
                                newline,
                            )),
                            |o| o.flatten(),
                        ),
                        map(
                            opt(delimited(
                                tag("["),
                                not_line_ending,
                                newline,
                            )),
                            |o| o.is_some(),
                        ),
                    )),
                    |(
                        interface,
                        ip_address,
                        host_name,
                        last_update,
                        update_status,
                        _,
                    )| {
                        DdnsStatus {
                            interface,
                            ip_address,
                            host_name,
                            last_update,
                            update_status,
                        }
                    },
                ),
                many1(line_ending),
            ),
        ),
    ))(input)
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use chrono::NaiveDate;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn empty() {
        let parser = DdnsParser;
        let input = "";

        assert!(parser.parse(input).is_err());
    }

    #[test]
    fn no_config() {
        let parser = DdnsParser;
        let input = indoc! {"
            Dynamic DNS not configured

        "};

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, vec![]);
    }

    #[test]
    fn no_statuses() {
        let parser = DdnsParser;
        let input = indoc! {"
            interface    : eth0 
            [ Status will be updated within 60 seconds ]

            interface    : eth1 
            [ Status will be updated within 60 seconds ]

        "};

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, vec![
            DdnsStatus {
                interface: "eth0".to_string(),
                ip_address: None,
                host_name: None,
                last_update: None,
                update_status: None,
            },
            DdnsStatus {
                interface: "eth1".to_string(),
                ip_address: None,
                host_name: None,
                last_update: None,
                update_status: None,
            },
        ]);
    }

    #[test]
    fn statuses() {
        let parser = DdnsParser;
        let input = indoc! {"
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

        let actual = parser.parse(input).unwrap();
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
