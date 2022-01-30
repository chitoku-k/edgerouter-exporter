use std::str::FromStr;

use anyhow::anyhow;
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_till, take_while},
    character::complete::{multispace1, newline, space0, space1, u64},
    combinator::{flat_map, map, map_parser, map_res, peek},
    error::Error,
    multi::many0,
    sequence::{delimited, terminated, tuple},
    Finish,
};

use crate::{
    domain::{
        interface::Interface,
        pppoe::{ByteSize, PPPoEClientSession, PacketSize},
    },
    infrastructure::cmd::parser::{parse_duration, Parser},
    service::pppoe::PPPoEClientSessionResult,
};

#[derive(Clone)]
pub struct PPPoEParser;

impl Parser for PPPoEParser {
    type Input = (String, Vec<Interface>);
    type Item = PPPoEClientSessionResult;

    fn parse(&self, (input, interfaces): Self::Input) -> anyhow::Result<Self::Item> {
        parse_pppoe_client_sessions(&input, &interfaces)
    }
}

fn parse_pppoe_client_sessions(input: &str, interfaces: &[Interface]) -> anyhow::Result<PPPoEClientSessionResult> {
    match alt((
        map(tag("No active PPPoE client sessions"), |_| vec![]),
        delimited(
            tuple((
                tag("Active PPPoE client sessions:"),
                multispace1,
                tag("User"),
                space1,
                tag("Time"),
                space1,
                tag("Proto"),
                space1,
                tag("Iface"),
                space1,
                tag("Remote IP"),
                space1,
                tag("TX pkt/byte"),
                space1,
                tag("RX pkt/byte"),
                multispace1,
                take_while(|c| c == '-' || c == ' '),
                multispace1,
            )),
            many0(
                map(
                    tuple((
                        terminated(
                            map(take_till(|c| c == ' '), &str::to_string),
                            space1,
                        ),
                        terminated(
                            map_parser(take_till(|c| c == ' '), parse_duration),
                            space1,
                        ),
                        terminated(
                            map(take_till(|c| c == ' '), &str::to_string),
                            space1,
                        ),
                        peek(terminated(
                            map(take_till(|c| c == ' '), &str::to_string),
                            space1,
                        )),
                        flat_map(
                            terminated(
                                map(take_till(|c| c == ' '), &str::to_string),
                                space1,
                            ),
                            |interface_name| {
                                let addr = interfaces.iter()
                                    .find(|i| i.ifname == interface_name)
                                    .map(|i| &i.addr_info)
                                    .and_then(|a| a.first());

                                let local_ip = addr.map(|a| a.local);
                                let remote_ip = addr
                                    .and_then(|i| i.address)
                                    .map(|a| a.to_string())
                                    .unwrap_or_default();

                                map(
                                    alt((
                                        terminated(
                                            map_res(take(remote_ip.len()), &str::parse),
                                            space0,
                                        ),
                                        terminated(
                                            map_res(take_till(|c| c == ' '), &str::parse),
                                            space1,
                                        ),
                                    )),
                                    move |remote_ip| {
                                        (remote_ip, local_ip)
                                    },
                                )
                            },
                        ),
                        terminated(
                            map_res(take_till(|c| c == ' '), PacketSize::from_str),
                            space1,
                        ),
                        terminated(
                            map_res(take_till(|c| c == ' '), ByteSize::from_str),
                            space1,
                        ),
                        terminated(
                            map_res(take_till(|c| c == ' '), PacketSize::from_str),
                            space1,
                        ),
                        terminated(
                            map_res(take_till(|c| c == '\n'), ByteSize::from_str),
                            newline,
                        ),
                    )),
                    |(
                        user,
                        time,
                        protocol,
                        interface,
                        (
                            remote_ip,
                            local_ip,
                        ),
                        transmit_packets,
                        transmit_bytes,
                        receive_packets,
                        receive_bytes,
                    )| {
                        PPPoEClientSession {
                            user,
                            time,
                            protocol,
                            interface,
                            remote_ip,
                            local_ip,
                            transmit_packets,
                            transmit_bytes,
                            receive_packets,
                            receive_bytes,
                        }
                    },
                ),
            ),
            tuple((
                multispace1,
                tag("Total sessions:"),
                space1,
                u64,
                multispace1,
            )),
        ),
    ))(input).finish() {
        Ok((_, sessions)) => Ok(sessions),
        Err::<_, Error<_>>(e) => Err(anyhow!(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr},
        time::Duration,
    };

    use indoc::indoc;
    use number_prefix::{NumberPrefix, Prefix};
    use pretty_assertions::assert_eq;

    use crate::domain::interface::AddrInfo;

    use super::*;

    #[test]
    fn empty() {
        let parser = PPPoEParser;
        let input = "";
        let interfaces = vec![];

        assert!(parser.parse((input.to_string(), interfaces)).is_err());
    }

    #[test]
    fn no_config() {
        let parser = PPPoEParser;
        let input = "No active PPPoE client sessions";
        let interfaces = vec![];

        let actual = parser.parse((input.to_string(), interfaces)).unwrap();
        assert_eq!(actual, vec![]);
    }

    #[test]
    fn sessions_without_interfaces() {
        let parser = PPPoEParser;
        let input = indoc! {"
            Active PPPoE client sessions:

            User       Time      Proto Iface   Remote IP       TX pkt/byte   RX pkt/byte
            ---------- --------- ----- -----   --------------- ------ ------ ------ ------
            user01     01h02m03s PPPoE pppoe0  192.0.2.255   384  34.8K   1.2K  58.2K
            user02     04d05h06m PPPoE pppoe1  198.51.100.255   768  76.8K   2.4K 116.4K

            Total sessions: 2
        "};
        let interfaces = vec![];

        let actual = parser.parse((input.to_string(), interfaces)).unwrap();
        assert_eq!(actual, vec![
            PPPoEClientSession {
                user: "user01".to_string(),
                time: Duration::new(3723, 0),
                protocol: "PPPoE".to_string(),
                interface: "pppoe0".to_string(),
                remote_ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 255)),
                local_ip: None,
                transmit_packets: NumberPrefix::Standalone(384.0).into(),
                transmit_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 34.8).into(),
                receive_packets: NumberPrefix::Prefixed(Prefix::Kilo, 1.2).into(),
                receive_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 58.2).into(),
            },
            PPPoEClientSession {
                user: "user02".to_string(),
                time: Duration::new(363960, 0),
                protocol: "PPPoE".to_string(),
                interface: "pppoe1".to_string(),
                remote_ip: IpAddr::V4(Ipv4Addr::new(198, 51, 100, 255)),
                local_ip: None,
                transmit_packets: NumberPrefix::Standalone(768.0).into(),
                transmit_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 76.8).into(),
                receive_packets: NumberPrefix::Prefixed(Prefix::Kilo, 2.4).into(),
                receive_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 116.4).into(),
            },
        ]);
    }

    #[test]
    fn sessions_with_interfaces() {
        let parser = PPPoEParser;
        let input = indoc! {"
            Active PPPoE client sessions:

            User       Time      Proto Iface   Remote IP       TX pkt/byte   RX pkt/byte
            ---------- --------- ----- -----   --------------- ------ ------ ------ ------
            user01     01h02m03s PPPoE pppoe0  192.0.2.255   384  34.8K   1.2K  58.2K
            user02     04d05h06m PPPoE pppoe1  198.51.100.255   768  76.8K   2.4K 116.4K

            Total sessions: 2
        "};
        let interfaces = vec![
            Interface {
                ifindex: 0,
                ifname: "pppoe0".to_string(),
                link: None,
                flags: vec![
                    "POINTOPOINT".to_string(),
                    "MULTICAST".to_string(),
                    "NOARP".to_string(),
                    "UP".to_string(),
                    "LOWER_UP".to_string(),
                ],
                mtu: 1454,
                qdisc: "pfifo_fast".to_string(),
                operstate: "UNKNOWN".to_string(),
                group: "default".to_string(),
                txqlen: 100,
                link_type: "ppp".to_string(),
                address: None,
                link_pointtopoint: None,
                broadcast: None,
                addr_info: vec![
                    AddrInfo {
                        family: "inet".to_string(),
                        local: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)),
                        address: Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 255))),
                        prefixlen: 32,
                        broadcast: None,
                        scope: "global".to_string(),
                        dynamic: None,
                        mngtmpaddr: None,
                        noprefixroute: None,
                        label: Some("pppoe0".to_string()),
                        valid_life_time: 4294967295,
                        preferred_life_time: 4294967295,
                    },
                ],
            },
            Interface {
                ifindex: 0,
                ifname: "pppoe1".to_string(),
                link: None,
                flags: vec![
                    "POINTOPOINT".to_string(),
                    "MULTICAST".to_string(),
                    "NOARP".to_string(),
                    "UP".to_string(),
                    "LOWER_UP".to_string(),
                ],
                mtu: 1454,
                qdisc: "pfifo_fast".to_string(),
                operstate: "UNKNOWN".to_string(),
                group: "default".to_string(),
                txqlen: 100,
                link_type: "ppp".to_string(),
                address: None,
                link_pointtopoint: None,
                broadcast: None,
                addr_info: vec![
                    AddrInfo {
                        family: "inet".to_string(),
                        local: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 2)),
                        address: Some(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 255))),
                        prefixlen: 32,
                        broadcast: None,
                        scope: "global".to_string(),
                        dynamic: None,
                        mngtmpaddr: None,
                        noprefixroute: None,
                        label: Some("pppoe1".to_string()),
                        valid_life_time: 4294967295,
                        preferred_life_time: 4294967295,
                    },
                ],
            },
        ];

        let actual = parser.parse((input.to_string(), interfaces)).unwrap();
        assert_eq!(actual, vec![
            PPPoEClientSession {
                user: "user01".to_string(),
                time: Duration::new(3723, 0),
                protocol: "PPPoE".to_string(),
                interface: "pppoe0".to_string(),
                remote_ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 255)),
                local_ip: Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1))),
                transmit_packets: NumberPrefix::Standalone(384.0).into(),
                transmit_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 34.8).into(),
                receive_packets: NumberPrefix::Prefixed(Prefix::Kilo, 1.2).into(),
                receive_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 58.2).into(),
            },
            PPPoEClientSession {
                user: "user02".to_string(),
                time: Duration::new(363960, 0),
                protocol: "PPPoE".to_string(),
                interface: "pppoe1".to_string(),
                remote_ip: IpAddr::V4(Ipv4Addr::new(198, 51, 100, 255)),
                local_ip: Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 2))),
                transmit_packets: NumberPrefix::Standalone(768.0).into(),
                transmit_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 76.8).into(),
                receive_packets: NumberPrefix::Prefixed(Prefix::Kilo, 2.4).into(),
                receive_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 116.4).into(),
            },
        ]);
    }

    #[test]
    fn sessions_ill_formed_with_interfaces() {
        let parser = PPPoEParser;
        let input = indoc! {"
            Active PPPoE client sessions:

            User       Time      Proto Iface   Remote IP       TX pkt/byte   RX pkt/byte
            ---------- --------- ----- -----   --------------- ------ ------ ------ ------
            user01     01h02m03s PPPoE pppoe0  192.0.2.255384  34.8K   1.2K  58.2K
            user02     04d05h06m PPPoE pppoe1  198.51.100.255768  76.8K   2.4K 116.4K

            Total sessions: 2
        "};
        let interfaces = vec![
            Interface {
                ifindex: 0,
                ifname: "pppoe0".to_string(),
                link: None,
                flags: vec![
                    "POINTOPOINT".to_string(),
                    "MULTICAST".to_string(),
                    "NOARP".to_string(),
                    "UP".to_string(),
                    "LOWER_UP".to_string(),
                ],
                mtu: 1454,
                qdisc: "pfifo_fast".to_string(),
                operstate: "UNKNOWN".to_string(),
                group: "default".to_string(),
                txqlen: 100,
                link_type: "ppp".to_string(),
                address: None,
                link_pointtopoint: None,
                broadcast: None,
                addr_info: vec![
                    AddrInfo {
                        family: "inet".to_string(),
                        local: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)),
                        address: Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 255))),
                        prefixlen: 32,
                        broadcast: None,
                        scope: "global".to_string(),
                        dynamic: None,
                        mngtmpaddr: None,
                        noprefixroute: None,
                        label: Some("pppoe0".to_string()),
                        valid_life_time: 4294967295,
                        preferred_life_time: 4294967295,
                    },
                ],
            },
            Interface {
                ifindex: 0,
                ifname: "pppoe1".to_string(),
                link: None,
                flags: vec![
                    "POINTOPOINT".to_string(),
                    "MULTICAST".to_string(),
                    "NOARP".to_string(),
                    "UP".to_string(),
                    "LOWER_UP".to_string(),
                ],
                mtu: 1454,
                qdisc: "pfifo_fast".to_string(),
                operstate: "UNKNOWN".to_string(),
                group: "default".to_string(),
                txqlen: 100,
                link_type: "ppp".to_string(),
                address: None,
                link_pointtopoint: None,
                broadcast: None,
                addr_info: vec![
                    AddrInfo {
                        family: "inet".to_string(),
                        local: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 2)),
                        address: Some(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 255))),
                        prefixlen: 32,
                        broadcast: None,
                        scope: "global".to_string(),
                        dynamic: None,
                        mngtmpaddr: None,
                        noprefixroute: None,
                        label: Some("pppoe1".to_string()),
                        valid_life_time: 4294967295,
                        preferred_life_time: 4294967295,
                    },
                ],
            },
        ];

        let actual = parser.parse((input.to_string(), interfaces)).unwrap();
        assert_eq!(actual, vec![
            PPPoEClientSession {
                user: "user01".to_string(),
                time: Duration::new(3723, 0),
                protocol: "PPPoE".to_string(),
                interface: "pppoe0".to_string(),
                remote_ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 255)),
                local_ip: Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1))),
                transmit_packets: NumberPrefix::Standalone(384.0).into(),
                transmit_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 34.8).into(),
                receive_packets: NumberPrefix::Prefixed(Prefix::Kilo, 1.2).into(),
                receive_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 58.2).into(),
            },
            PPPoEClientSession {
                user: "user02".to_string(),
                time: Duration::new(363960, 0),
                protocol: "PPPoE".to_string(),
                interface: "pppoe1".to_string(),
                remote_ip: IpAddr::V4(Ipv4Addr::new(198, 51, 100, 255)),
                local_ip: Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 2))),
                transmit_packets: NumberPrefix::Standalone(768.0).into(),
                transmit_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 76.8).into(),
                receive_packets: NumberPrefix::Prefixed(Prefix::Kilo, 2.4).into(),
                receive_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 116.4).into(),
            },
        ]);
    }
}
