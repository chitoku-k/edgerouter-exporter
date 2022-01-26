use std::str::FromStr;

use anyhow::anyhow;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_while},
    character::complete::{multispace1, newline, space1, u64},
    combinator::{map, map_parser, map_res},
    error::Error,
    multi::many0,
    sequence::{delimited, terminated, tuple},
    Finish,
};

use crate::{
    domain::pppoe::{ByteSize, PPPoEClientSession, PacketSize},
    infrastructure::cmd::parser::{parse_duration, Parser},
    service::pppoe::PPPoEClientSessionResult,
};

#[derive(Clone)]
pub struct PPPoEParser;

impl Parser for PPPoEParser {
    type Item = PPPoEClientSessionResult;

    fn parse(&self, input: &str) -> anyhow::Result<Self::Item> {
        parse_pppoe_client_sessions(input)
    }
}

fn parse_pppoe_client_sessions(input: &str) -> anyhow::Result<PPPoEClientSessionResult> {
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
                        terminated(
                            map(take_till(|c| c == ' '), &str::to_string),
                            space1,
                        ),
                        terminated(
                            map_res(take_till(|c| c == ' '), &str::parse),
                            space1,
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
                        remote_ip,
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

    use super::*;

    #[test]
    fn empty() {
        let parser = PPPoEParser;
        let input = "";

        assert!(parser.parse(input).is_err());
    }

    #[test]
    fn no_config() {
        let parser = PPPoEParser;
        let input = "No active PPPoE client sessions";

        let actual = parser.parse(input);
        assert!(actual.is_ok());

        let actual = actual.unwrap();
        assert_eq!(actual, vec![]);
    }

    #[test]
    fn sessions() {
        let parser = PPPoEParser;
        let input = indoc! {"
            Active PPPoE client sessions:

            User       Time      Proto Iface   Remote IP       TX pkt/byte   RX pkt/byte
            ---------- --------- ----- -----   --------------- ------ ------ ------ ------
            user01     01h02m03s PPPoE pppoe0  192.0.2.255   384  34.8K   1.2K  58.2K
            user02     04d05h06m PPPoE pppoe1  198.51.100.255   768  76.8K   2.4K 116.4K

            Total sessions: 2
        "};

        let actual = parser.parse(input);
        assert!(actual.is_ok());

        let actual = actual.unwrap();
        assert_eq!(actual, vec![
            PPPoEClientSession {
                user: "user01".to_string(),
                time: Duration::new(3723, 0),
                protocol: "PPPoE".to_string(),
                interface: "pppoe0".to_string(),
                remote_ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 255)),
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
                transmit_packets: NumberPrefix::Standalone(768.0).into(),
                transmit_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 76.8).into(),
                receive_packets: NumberPrefix::Prefixed(Prefix::Kilo, 2.4).into(),
                receive_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 116.4).into(),
            },
        ]);
    }
}
