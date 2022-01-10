use std::collections::HashMap;

use anyhow::{anyhow, Result};
use nom::{
    branch::permutation,
    bytes::complete::{tag, take_till, take_until},
    character::complete::{multispace1, newline, space1, u32, u64},
    combinator::{map, map_res, opt},
    error::Error,
    multi::{many0, many1, many_till, separated_list1},
    sequence::{delimited, terminated, tuple},
    Finish,
};

use crate::{
    domain::bgp::{BGPNeighbor, BGPStatus},
    infrastructure::cmd::parser::{parse_duration, Parser},
};

pub struct BGPParser;

impl Parser for BGPParser {
    type Item = Option<BGPStatus>;

    fn parse(&self, input: &str) -> Result<Self::Item> {
        parse_bgp_status(input)
    }
}

fn parse_bgp_neighbor(header: &[&str], entry: &[&str]) -> Result<BGPNeighbor> {
    let entry: HashMap<_, _> = header.iter().cloned().zip(entry.iter().cloned()).collect();

    Ok(BGPNeighbor {
        neighbor: entry.get("Neighbor").ok_or_else(|| anyhow!("cannot find neighbor"))?.parse()?,
        version: entry.get("V").ok_or_else(|| anyhow!("cannot find version"))?.parse()?,
        remote_as: entry.get("AS").ok_or_else(|| anyhow!("cannot find remote_as"))?.parse()?,
        messages_received: entry.get("MsgRcv").ok_or_else(|| anyhow!("cannot find messages_received"))?.parse()?,
        messages_sent: entry.get("MsgSen").ok_or_else(|| anyhow!("cannot find messages_sent"))?.parse()?,
        table_version: entry.get("TblVer").ok_or_else(|| anyhow!("cannot find table_version"))?.parse()?,
        in_queue: entry.get("InQ").ok_or_else(|| anyhow!("cannot find in_queue"))?.parse()?,
        out_queue: entry.get("OutQ").ok_or_else(|| anyhow!("cannot find out_queue"))?.parse()?,
        uptime: entry.get("Up/Down").and_then(|v| parse_duration(v).ok()).map(|(_, u)| u),
        state: entry.get("State/PfxRcd").filter(|v| v.chars().all(|c| !c.is_ascii_digit())).map(|v| v.to_string()),
        prefixes_received: entry.get("State/PfxRcd").and_then(|v| v.parse().ok()),
    })
}

fn parse_bgp_status(input: &str) -> Result<Option<BGPStatus>> {
    match opt(
        map(
            permutation((
                tuple((
                    delimited(
                        tuple((tag("BGP router identifier"), space1)),
                        map(take_until(","), &str::to_string),
                        tuple((tag(","), space1)),
                    ),
                    delimited(
                        tuple((tag("local AS number"), space1)),
                        u32,
                        many1(newline),
                    ),
                )),
                delimited(
                    tuple((tag("BGP table version is"), space1)),
                    u32,
                    many1(newline),
                ),
                terminated(
                    u64,
                    tuple((space1, tag("BGP AS-PATH entries"), many1(newline))),
                ),
                terminated(
                    u64,
                    tuple((space1, tag("BGP community entries"), many1(newline))),
                ),
                map_res(
                    many_till(
                        terminated(
                            separated_list1(space1, take_till(|c| c == ' ' || c == '\n')),
                            newline,
                        ),
                        multispace1,
                    ),
                    |(lines, _)| {
                        lines.split_first()
                            .ok_or_else(|| anyhow!("cannot find header"))
                            .and_then(|(header, values)| {
                                values.iter().map(|v| parse_bgp_neighbor(header, v)).collect()
                            })
                    },
                ),
                delimited(
                    tuple((tag("Total number of neighbors"), space1)),
                    u64,
                    many0(newline),
                ),
                delimited(
                    tuple((tag("Total number of Established sessions"), space1)),
                    u64,
                    many0(newline),
                ),
            )),
            |(
                (router_id, local_as),
                table_version,
                as_paths,
                communities,
                neighbors,
                _,
                sessions,
            )| {
                BGPStatus {
                    router_id,
                    local_as,
                    table_version,
                    as_paths,
                    communities,
                    neighbors,
                    sessions,
                }
            },
        ),
    )(input).finish() {
        Ok((_, status)) => Ok(status),
        Err::<_, Error<_>>(e) => Err(anyhow!(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr, Ipv6Addr},
        time::Duration,
    };

    use cool_asserts::assert_matches;
    use indoc::indoc;

    use super::*;

    #[test]
    fn empty() {
        let parser = BGPParser;
        let input = "";

        assert_matches!(
            parser.parse(input),
            Ok(None),
        );
    }

    #[test]
    fn neighbors() {
        let parser = BGPParser;
        let input = indoc! {"
            BGP router identifier 192.0.2.1, local AS number 64496
            BGP table version is 128
            1 BGP AS-PATH entries
            2 BGP community entries

            Neighbor                 V   AS   MsgRcv    MsgSen TblVer   InQ   OutQ    Up/Down   State/PfxRcd
            192.0.2.2                4 64497 1000       5000     128      1      5  01:11:11               9
            192.0.2.3                4 64497 2000       6000     128      2      6  1d02h03m              10
            192.0.2.4                4 64497    0          0       0      0      0     never     Connect
            2001:db8::2              4 64497 3000       7000     128      3      7  03:33:33               0
            2001:db8::3              4 64497 4000       8000     128      4      8  4d05h06m               0
            2001:db8::4              4 64497    0          0       0      0      0     never     Connect

            Total number of neighbors 6

            Total number of Established sessions 4
        "};

        assert_matches!(
            parser.parse(input),
            Ok(status) if status == Some(BGPStatus {
                router_id: "192.0.2.1".to_string(),
                local_as: 64496,
                table_version: 128,
                as_paths: 1,
                communities: 2,
                neighbors: vec![
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 2)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 1000,
                        messages_sent: 5000,
                        table_version: 128,
                        in_queue: 1,
                        out_queue: 5,
                        uptime: Some(Duration::new(4271, 0)),
                        state: None,
                        prefixes_received: Some(9),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 3)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 2000,
                        messages_sent: 6000,
                        table_version: 128,
                        in_queue: 2,
                        out_queue: 6,
                        uptime: Some(Duration::new(93780, 0)),
                        state: None,
                        prefixes_received: Some(10),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 4)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 0,
                        messages_sent: 0,
                        table_version: 0,
                        in_queue: 0,
                        out_queue: 0,
                        uptime: None,
                        state: Some("Connect".to_string()),
                        prefixes_received: None,
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x2)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 3000,
                        messages_sent: 7000,
                        table_version: 128,
                        in_queue: 3,
                        out_queue: 7,
                        uptime: Some(Duration::new(12813, 0)),
                        state: None,
                        prefixes_received: Some(0),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x3)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 4000,
                        messages_sent: 8000,
                        table_version: 128,
                        in_queue: 4,
                        out_queue: 8,
                        uptime: Some(Duration::new(363960, 0)),
                        state: None,
                        prefixes_received: Some(0),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x4)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 0,
                        messages_sent: 0,
                        table_version: 0,
                        in_queue: 0,
                        out_queue: 0,
                        uptime: None,
                        state: Some("Connect".to_string()),
                        prefixes_received: None,
                    },
                ],
                sessions: 4,
            }),
        );
    }
}
