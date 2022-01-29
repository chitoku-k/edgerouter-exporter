use crate::{
    domain::interface::Interface,
    infrastructure::cmd::parser::Parser,
    service::interface::InterfaceResult,
};

#[derive(Clone)]
pub struct InterfaceParser;

impl Parser for InterfaceParser {
    type Item = InterfaceResult;

    fn parse(&self, input: &str) -> anyhow::Result<Self::Item> {
        parse_interface(input)
    }
}

fn parse_interface(input: &str) -> anyhow::Result<Vec<Interface>> {
    let interfaces = serde_json::from_str(input)?;
    Ok(interfaces)
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
            [{
                    "ifindex": 1,
                    "ifname": "lo",
                    "flags": ["LOOPBACK","UP","LOWER_UP"],
                    "mtu": 65536,
                    "qdisc": "noqueue",
                    "operstate": "UNKNOWN",
                    "group": "default",
                    "txqlen": 1000,
                    "link_type": "loopback",
                    "address": "00:00:00:00:00:00",
                    "broadcast": "00:00:00:00:00:00",
                    "addr_info": [{
                            "family": "inet",
                            "local": "127.0.0.1",
                            "prefixlen": 8,
                            "scope": "host",
                            "label": "lo",
                            "valid_life_time": 4294967295,
                            "preferred_life_time": 4294967295
                        },{
                            "family": "inet6",
                            "local": "::1",
                            "prefixlen": 128,
                            "scope": "host",
                            "valid_life_time": 4294967295,
                            "preferred_life_time": 4294967295
                        }]
                }
            ]
        "#};

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, vec![
            Interface {
                ifindex: 1,
                ifname: "lo".to_string(),
                link: None,
                flags: vec![
                    "LOOPBACK".to_string(),
                    "UP".to_string(),
                    "LOWER_UP".to_string(),
                ],
                mtu: 65536,
                qdisc: "noqueue".to_string(),
                operstate: "UNKNOWN".to_string(),
                group: "default".to_string(),
                txqlen: 1000,
                link_type: "loopback".to_string(),
                address: Some("00:00:00:00:00:00".to_string()),
                link_pointtopoint: None,
                broadcast: Some("00:00:00:00:00:00".to_string()),
                addr_info: vec![
                    AddrInfo {
                        family: "inet".to_string(),
                        local: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                        address: None,
                        prefixlen: 8,
                        broadcast: None,
                        scope: "host".to_string(),
                        dynamic: None,
                        mngtmpaddr: None,
                        noprefixroute: None,
                        label: Some("lo".to_string()),
                        valid_life_time: 4294967295,
                        preferred_life_time: 4294967295,
                    },
                    AddrInfo {
                        family: "inet6".to_string(),
                        local: IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
                        address: None,
                        prefixlen: 128,
                        broadcast: None,
                        scope: "host".to_string(),
                        dynamic: None,
                        mngtmpaddr: None,
                        noprefixroute: None,
                        label: None,
                        valid_life_time: 4294967295,
                        preferred_life_time: 4294967295,
                    },
                ],
            },
        ]);
    }
}
