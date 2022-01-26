use std::{
    iter::{Chain, Flatten},
    net::IpAddr,
    option::IntoIter,
    time::Duration,
};

use derive_more::{Deref, DerefMut, IntoIterator};

#[derive(Deref, DerefMut, IntoIterator)]
pub struct BGPIterator(Chain<
    Flatten<IntoIter<Vec<BGPNeighbor>>>,
    Flatten<IntoIter<Vec<BGPNeighbor>>>,
>);

#[derive(Clone, Debug, PartialEq)]
pub struct BGPStatus {
    pub router_id: String,
    pub local_as: u32,
    pub table_version: u32,
    pub as_paths: u64,
    pub communities: u64,
    pub neighbors: Vec<BGPNeighbor>,
    pub sessions: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BGPNeighbor {
    pub neighbor: IpAddr,
    pub version: u32,
    pub remote_as: u32,
    pub messages_received: u64,
    pub messages_sent: u64,
    pub table_version: u32,
    pub in_queue: u64,
    pub out_queue: u64,
    pub uptime: Option<Duration>,
    pub state: Option<String>,
    pub prefixes_received: Option<u64>,
}

impl From<(Option<BGPStatus>, Option<BGPStatus>)> for BGPIterator {
    fn from((bgp4, bgp6): (Option<BGPStatus>, Option<BGPStatus>)) -> Self {
        Self(
            Iterator::chain(
                bgp4.map(|s| s.neighbors).into_iter().flatten(),
                bgp6.map(|s| s.neighbors).into_iter().flatten(),
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn iterate_empty() {
        let status = (None, None);

        let mut iterator = BGPIterator::from(status);
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn iterate_all() {
        let status = (
            Some(BGPStatus {
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
                ],
                sessions: 3,
            }),
            Some(BGPStatus {
                router_id: "192.0.2.1".to_string(),
                local_as: 64496,
                table_version: 128,
                as_paths: 1,
                communities: 2,
                neighbors: vec![
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

        let mut iterator = BGPIterator::from(status);
        assert_eq!(
            iterator.next(),
            Some(BGPNeighbor {
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
            }),
        );
        assert_eq!(
            iterator.next(),
            Some(BGPNeighbor {
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
            }),
        );
        assert_eq!(
            iterator.next(),
            Some(BGPNeighbor {
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
            }),
        );
        assert_eq!(
            iterator.next(),
            Some(BGPNeighbor {
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
            }),
        );
        assert_eq!(
            iterator.next(),
            Some(BGPNeighbor {
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
            }),
        );
        assert_eq!(
            iterator.next(),
            Some(BGPNeighbor {
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
            }),
        );
        assert_eq!(iterator.next(), None);
    }
}
