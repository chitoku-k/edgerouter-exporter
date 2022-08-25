use std::net::IpAddr;

#[derive(Clone, Debug, PartialEq)]
pub struct Interface {
    pub ifname: String,
    pub operstate: String,
    pub addr_info: Vec<AddrInfo>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AddrInfo {
    pub local: IpAddr,
    pub address: Option<IpAddr>,
    pub prefixlen: u32,
}
