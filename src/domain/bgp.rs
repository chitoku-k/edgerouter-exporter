use std::{net::IpAddr, time::Duration};

#[derive(Debug, PartialEq)]
pub struct BGPStatus {
    pub router_id: String,
    pub local_as: u32,
    pub table_version: u32,
    pub as_paths: u64,
    pub communities: u64,
    pub neighbors: Vec<BGPNeighbor>,
    pub sessions: u64,
}

#[derive(Debug, PartialEq)]
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
