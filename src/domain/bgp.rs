use std::{
    iter::{Chain, Flatten},
    net::IpAddr,
    option::IntoIter,
    time::Duration,
};

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

impl Iterator for BGPIterator {
    type Item = BGPNeighbor;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
