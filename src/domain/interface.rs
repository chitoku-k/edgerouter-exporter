use std::net::IpAddr;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Interface {
    pub ifindex: u32,
    pub ifname: String,
    pub link: Option<String>,
    pub flags: Vec<String>,
    pub mtu: u32,
    pub qdisc: String,
    pub operstate: String,
    pub group: String,
    pub txqlen: u32,
    pub link_type: String,
    pub address: Option<String>,
    pub link_pointtopoint: Option<bool>,
    pub broadcast: Option<String>,
    pub addr_info: Vec<AddrInfo>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AddrInfo {
    pub family: String,
    pub local: IpAddr,
    pub address: Option<IpAddr>,
    pub prefixlen: u32,
    pub broadcast: Option<IpAddr>,
    pub scope: String,
    pub dynamic: Option<bool>,
    pub mngtmpaddr: Option<bool>,
    pub noprefixroute: Option<bool>,
    pub label: Option<String>,
    pub valid_life_time: u64,
    pub preferred_life_time: u64,
}
