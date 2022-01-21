use std::{net::IpAddr, time::Duration};

use derive_more::{From, FromStr};
use number_prefix::{NumberPrefix, Prefix};

const NUM_1024: f64 = 1024_f64;

#[derive(Clone, Debug, From, FromStr, PartialEq)]
pub struct PacketSize(NumberPrefix<f32>);

#[derive(Clone, Debug, From, FromStr, PartialEq)]
pub struct ByteSize(NumberPrefix<f32>);

#[derive(Clone, Debug, PartialEq)]
pub struct PPPoEClientSession {
    pub user: String,
    pub time: Duration,
    pub protocol: String,
    pub interface: String,
    pub remote_ip: IpAddr,
    pub transmit_packets: PacketSize,
    pub transmit_bytes: ByteSize,
    pub receive_packets: PacketSize,
    pub receive_bytes: ByteSize,
}

fn convert_size(prefix: NumberPrefix<f32>) -> u64 {
    let n = match prefix {
        NumberPrefix::Standalone(v) => v as f64,

        NumberPrefix::Prefixed(Prefix::Kilo, v) => v as f64 * NUM_1024.powi(1),
        NumberPrefix::Prefixed(Prefix::Mega, v) => v as f64 * NUM_1024.powi(2),
        NumberPrefix::Prefixed(Prefix::Giga, v) => v as f64 * NUM_1024.powi(3),
        NumberPrefix::Prefixed(Prefix::Tera, v) => v as f64 * NUM_1024.powi(4),
        NumberPrefix::Prefixed(Prefix::Peta, v) => v as f64 * NUM_1024.powi(5),
        NumberPrefix::Prefixed(Prefix::Exa, v) => v as f64 * NUM_1024.powi(6),
        NumberPrefix::Prefixed(Prefix::Zetta, v) => v as f64 * NUM_1024.powi(7),
        NumberPrefix::Prefixed(Prefix::Yotta, v) => v as f64 * NUM_1024.powi(8),

        NumberPrefix::Prefixed(Prefix::Kibi, v) => v as f64 * NUM_1024.powi(1),
        NumberPrefix::Prefixed(Prefix::Mebi, v) => v as f64 * NUM_1024.powi(2),
        NumberPrefix::Prefixed(Prefix::Gibi, v) => v as f64 * NUM_1024.powi(3),
        NumberPrefix::Prefixed(Prefix::Tebi, v) => v as f64 * NUM_1024.powi(4),
        NumberPrefix::Prefixed(Prefix::Pebi, v) => v as f64 * NUM_1024.powi(5),
        NumberPrefix::Prefixed(Prefix::Exbi, v) => v as f64 * NUM_1024.powi(6),
        NumberPrefix::Prefixed(Prefix::Zebi, v) => v as f64 * NUM_1024.powi(7),
        NumberPrefix::Prefixed(Prefix::Yobi, v) => v as f64 * NUM_1024.powi(8),
    };

    n as u64
}

impl From<PacketSize> for u64 {
    fn from(prefix: PacketSize) -> Self {
        convert_size(prefix.0)
    }
}

impl From<ByteSize> for u64 {
    fn from(prefix: ByteSize) -> Self {
        convert_size(prefix.0)
    }
}
