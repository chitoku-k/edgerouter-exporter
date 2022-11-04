use std::net::IpAddr;

use anyhow::Context;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_while},
    character::complete::{multispace1, space0, space1, u32},
    combinator::{map, map_res},
    error::Error,
    multi::many0,
    sequence::{separated_pair, terminated, tuple},
    Finish, IResult,
};

use crate::{
    domain::interface::{AddrInfo, Interface},
    infrastructure::cmd::parser::Parser,
    service::interface::InterfaceResult,
};

#[derive(Clone)]
pub struct InterfaceParser;

impl Parser for InterfaceParser {
    type Input<'a> = &'a str;
    type Item = InterfaceResult;

    fn parse(&self, input: Self::Input<'_>) -> anyhow::Result<Self::Item> {
        parse_interfaces(input)
            .finish()
            .map(|(_, interfaces)| interfaces)
            .map_err(|e| Error::new(e.input.to_string(), e.code))
            .context("failed to parse interfaces")
    }
}

fn parse_cidr(input: &str) -> IResult<&str, (IpAddr, u32)> {
    separated_pair(
        parse_ip_address,
        tag("/"),
        u32,
    )(input)
}

fn parse_ip_address(input: &str) -> IResult<&str, IpAddr> {
    alt((
        map_res(take_while(|c| c == '.' || char::is_digit(c, 10)), &str::parse),
        map_res(take_while(|c| c == ':' || char::is_digit(c, 16)), &str::parse),
    ))(input)
}

fn parse_interfaces(input: &str) -> IResult<&str, InterfaceResult> {
    many0(
        terminated(
            map(
                tuple((
                    terminated(
                        map(take_till(|c| c == ' '), &str::to_string),
                        space1,
                    ),
                    terminated(
                        map(take_till(|c| c == ' '), &str::to_string),
                        space1,
                    ),
                    many0(
                        terminated(
                            alt((
                                map(
                                    separated_pair(
                                        parse_ip_address,
                                        tuple((space1, tag("peer"), space1)),
                                        parse_cidr,
                                    ),
                                    |(local, (address, prefixlen))| {
                                        AddrInfo {
                                            local,
                                            address: Some(address),
                                            prefixlen,
                                        }
                                    },
                                ),
                                map(
                                    parse_cidr,
                                    |(local, prefixlen)| {
                                        AddrInfo {
                                            local,
                                            address: None,
                                            prefixlen,
                                        }
                                    },
                                ),
                                map(
                                    parse_ip_address,
                                    |local| {
                                        let prefixlen = match local {
                                            IpAddr::V4(_) => 32,
                                            IpAddr::V6(_) => 128,
                                        };
                                        AddrInfo {
                                            local,
                                            address: None,
                                            prefixlen,
                                        }
                                    },
                                ),
                            )),
                            space0,
                        ),
                    ),
                )),
                |(ifname, operstate, addr_info)| {
                    Interface {
                        ifname,
                        operstate,
                        addr_info,
                    }
                },
            ),
            multispace1,
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr, IpAddr};

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use crate::domain::interface::AddrInfo;

    use super::*;

    #[test]
    fn interfaces() {
        let parser = InterfaceParser;
        let input = indoc! {r#"
            lo               UNKNOWN        127.0.0.1/8 ::1/128 
            imq0             DOWN           
            pppoe0           UP             203.0.113.1 peer 192.0.2.255/32 
        "#};

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, vec![
            Interface {
                ifname: "lo".to_string(),
                operstate: "UNKNOWN".to_string(),
                addr_info: vec![
                    AddrInfo {
                        local: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                        address: None,
                        prefixlen: 8,
                    },
                    AddrInfo {
                        local: IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
                        address: None,
                        prefixlen: 128,
                    },
                ],
            },
            Interface {
                ifname: "imq0".to_string(),
                operstate: "DOWN".to_string(),
                addr_info: vec![],
            },
            Interface {
                ifname: "pppoe0".to_string(),
                operstate: "UP".to_string(),
                addr_info: vec![
                    AddrInfo {
                        local: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)),
                        address: Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 255))),
                        prefixlen: 32,
                    },
                ],
            },
        ]);
    }
}
