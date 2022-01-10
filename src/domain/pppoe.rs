use std::{net::IpAddr, time::Duration};

use number_prefix::NumberPrefix;

#[derive(Debug, PartialEq)]
pub struct PPPoEClientSession {
    pub user: String,
    pub time: Duration,
    pub protocol: String,
    pub interface: String,
    pub remote_ip: IpAddr,
    pub transmit_packets: NumberPrefix<f32>,
    pub transmit_bytes: NumberPrefix<f32>,
    pub receive_packets: NumberPrefix<f32>,
    pub receive_bytes: NumberPrefix<f32>,
}
