use std::collections::HashMap;

use anyhow::Context;
use nom::{
    branch::{alt, permutation},
    bytes::complete::{tag, take_till, take_until},
    character::complete::{multispace0, multispace1, newline, not_line_ending, space1, u32, u64},
    combinator::{eof, map, map_res, opt},
    error::Error,
    multi::{many0, many1, many_till, separated_list1},
    sequence::{delimited, terminated, tuple},
    Finish, IResult,
};

use crate::{
    domain::bgp::{BGPNeighbor, BGPStatus},
    infrastructure::cmd::parser::{Duration, Parser},
    service::bgp::BGPStatusResult,
};

const BGP_VERSION: &str = "4";

pub struct BGPParser;

impl Parser for BGPParser {
    type Input<'a> = &'a str;
    type Item = BGPStatusResult;

    fn parse(&self, input: Self::Input<'_>) -> anyhow::Result<Self::Item> {
        parse_bgp_status(input)
            .finish()
            .map(|(_, status)| status)
            .map_err(|e| Error::new(e.input.to_string(), e.code))
            .context("failed to parse BGP status")
    }
}

fn parse_bgp_neighbor(header: &[&str], line: &[&str]) -> anyhow::Result<BGPNeighbor> {
    let entry: HashMap<_, _> = header.iter().zip(line.iter()).collect();
    if header.len() > entry.len() && entry.get(&"V").is_some_and(|&&v| v != BGP_VERSION) {
        return header
            .iter()
            .position(|&k| k == "V")
            .and_then(|i| line.get(i - 1).and_then(|v| v.strip_suffix(BGP_VERSION)).map(|v| (i, v))
            .map(|(i, v)| [&line[..i - 1], &[v], &[BGP_VERSION], &line[i..]].concat()))
            .map(|line| parse_bgp_neighbor(header, &line))
            .context("invalid V")?;
    }

    let state_or_pfx_rcd = entry.get(&"State/PfxRcd").context("cannot find State/PfxRcd")?;
    let (state, prefixes_received) = match state_or_pfx_rcd.parse().ok() {
        Some(prefixes_received) => (None, Some(prefixes_received)),
        None => (Some(state_or_pfx_rcd.to_string()), None),
    };

    Ok(BGPNeighbor {
        neighbor: entry.get(&"Neighbor").context("cannot find Neighbor")?
            .parse().context("cannot parse Neighbor")?,
        version: entry.get(&"V").context("cannot find V")?
            .parse().context("cannot parse V")?,
        remote_as: entry.get(&"AS").context("cannot find AS")?
            .parse().context("cannot parse AS")?,
        messages_received: entry.get(&"MsgRcv").context("cannot find MsgRcv")?
            .parse().context("cannot parse MsgRcv")?,
        messages_sent: entry.get(&"MsgSen").context("cannot find MsgSen")?
            .parse().context("cannot parse MsgSen")?,
        table_version: entry.get(&"TblVer").context("cannot find TblVer")?
            .parse().context("cannot parse TblVer")?,
        in_queue: entry.get(&"InQ").context("cannot find InQ")?
            .parse().context("cannot parse InQ")?,
        out_queue: entry.get(&"OutQ").context("cannot find OutQ")?
            .parse().context("cannot parse OutQ")?,
        uptime: entry.get(&"Up/Down").context("cannot find Up/Down")?
            .parse().ok().map(Duration::into),
        state,
        prefixes_received,
    })
}

fn parse_bgp_status(input: &str) -> IResult<&str, BGPStatusResult> {
    alt((
        map(
            tuple((multispace0, eof)),
            |_| None,
        ),
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
                opt(terminated(
                    u64,
                    tuple((space1, tag("Configured ebgp ECMP multipath"), not_line_ending, many1(newline))),
                )),
                opt(terminated(
                    u64,
                    tuple((space1, tag("Configured ibgp ECMP multipath"), not_line_ending, many1(newline))),
                )),
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
                            .context("cannot find header")
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
                ebgp_maximum_paths,
                ibgp_maximum_paths,
                neighbors,
                _,
                sessions,
            )| {
                Some(BGPStatus {
                    router_id,
                    local_as,
                    table_version,
                    as_paths,
                    communities,
                    ebgp_maximum_paths,
                    ibgp_maximum_paths,
                    neighbors,
                    sessions,
                })
            },
        ),
    ))(input)
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr, Ipv6Addr},
        time::Duration,
    };

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn invalid() {
        let parser = BGPParser;
        let input = "command not found";

        let actual = parser.parse(input);
        assert!(actual.is_err());
    }

    #[test]
    fn empty() {
        let parser = BGPParser;
        let input = "";

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, None);
    }

    #[test]
    fn neighbors_without_maximum_paths() {
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
            2001:db8::ffff:ffff:ffff:ffff4 64497    0          0       0      0      0     never     Connect

            Total number of neighbors 6

            Total number of Established sessions 4
        "};

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, Some(BGPStatus {
            router_id: "192.0.2.1".to_string(),
            local_as: 64496,
            table_version: 128,
            as_paths: 1,
            communities: 2,
            ebgp_maximum_paths: None,
            ibgp_maximum_paths: None,
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
                    neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0xffff, 0x0ffff, 0xffff, 0xffff)),
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
        }));
    }

    #[test]
    fn neighbors_with_maximum_paths() {
        let parser = BGPParser;
        let input = indoc! {"
            BGP router identifier 192.0.2.1, local AS number 64496
            BGP table version is 128
            1 BGP AS-PATH entries
            2 BGP community entries
            8  Configured ebgp ECMP multipath: Currently set at 8
            4  Configured ibgp ECMP multipath: Currently set at 4

            Neighbor                 V   AS   MsgRcv    MsgSen TblVer   InQ   OutQ    Up/Down   State/PfxRcd
            192.0.2.2                4 64497 1000       5000     128      1      5  01:11:11               9
            192.0.2.3                4 64497 2000       6000     128      2      6  1d02h03m              10
            192.0.2.4                4 64497    0          0       0      0      0     never     Connect
            2001:db8::2              4 64497 3000       7000     128      3      7  03:33:33               0
            2001:db8::3              4 64497 4000       8000     128      4      8  4d05h06m               0
            2001:db8::ffff:ffff:ffff:ffff4 64497    0          0       0      0      0     never     Connect

            Total number of neighbors 6

            Total number of Established sessions 4
        "};

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, Some(BGPStatus {
            router_id: "192.0.2.1".to_string(),
            local_as: 64496,
            table_version: 128,
            as_paths: 1,
            communities: 2,
            ebgp_maximum_paths: Some(8),
            ibgp_maximum_paths: Some(4),
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
                    neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0xffff, 0x0ffff, 0xffff, 0xffff)),
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
        }));
    }
}
