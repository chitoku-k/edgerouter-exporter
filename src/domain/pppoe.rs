use std::{net::IpAddr, time::Duration};

use derive_more::{From, FromStr};
use number_prefix::NumberPrefix;

use super::convert_size;

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
    pub local_ip: Option<IpAddr>,
    pub transmit_packets: PacketSize,
    pub transmit_bytes: ByteSize,
    pub receive_packets: PacketSize,
    pub receive_bytes: ByteSize,
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

#[cfg(test)]
mod tests {
    use std::panic;

    use number_prefix::Prefix;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn convert_packet_size() {
        assert_eq!(u64::from(PacketSize::from(NumberPrefix::Standalone(10_f32))), 10);

        assert_eq!(u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Kilo, 10_f32))), 10 * 1024);
        assert_eq!(u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Mega, 10_f32))), 10 * 1024 * 1024);
        assert_eq!(u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Giga, 10_f32))), 10 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Tera, 10_f32))), 10 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Peta, 10_f32))), 10 * 1024 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Exa, 10_f32))), 10 * 1024 * 1024 * 1024 * 1024 * 1024 * 1024);

        assert_eq!(u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Kibi, 10_f32))), 10 * 1024);
        assert_eq!(u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Mebi, 10_f32))), 10 * 1024 * 1024);
        assert_eq!(u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Gibi, 10_f32))), 10 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Tebi, 10_f32))), 10 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Pebi, 10_f32))), 10 * 1024 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Exbi, 10_f32))), 10 * 1024 * 1024 * 1024 * 1024 * 1024 * 1024);
    }

    #[test]
    fn convert_packet_size_overflow() {
        assert!(panic::catch_unwind(|| u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Yotta, 10_f32)))).is_err());
        assert!(panic::catch_unwind(|| u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Zetta, 10_f32)))).is_err());

        assert!(panic::catch_unwind(|| u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Yobi, 10_f32)))).is_err());
        assert!(panic::catch_unwind(|| u64::from(PacketSize::from(NumberPrefix::Prefixed(Prefix::Zebi, 10_f32)))).is_err());
    }

    #[test]
    fn convert_byte_size() {
        assert_eq!(u64::from(ByteSize::from(NumberPrefix::Standalone(10_f32))), 10);

        assert_eq!(u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Kilo, 10_f32))), 10 * 1024);
        assert_eq!(u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Mega, 10_f32))), 10 * 1024 * 1024);
        assert_eq!(u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Giga, 10_f32))), 10 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Tera, 10_f32))), 10 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Peta, 10_f32))), 10 * 1024 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Exa, 10_f32))), 10 * 1024 * 1024 * 1024 * 1024 * 1024 * 1024);

        assert_eq!(u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Kibi, 10_f32))), 10 * 1024);
        assert_eq!(u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Mebi, 10_f32))), 10 * 1024 * 1024);
        assert_eq!(u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Gibi, 10_f32))), 10 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Tebi, 10_f32))), 10 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Pebi, 10_f32))), 10 * 1024 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Exbi, 10_f32))), 10 * 1024 * 1024 * 1024 * 1024 * 1024 * 1024);
    }

    #[test]
    fn convert_byte_size_overflow() {
        assert!(panic::catch_unwind(|| u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Yotta, 10_f32)))).is_err());
        assert!(panic::catch_unwind(|| u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Zetta, 10_f32)))).is_err());

        assert!(panic::catch_unwind(|| u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Yobi, 10_f32)))).is_err());
        assert!(panic::catch_unwind(|| u64::from(ByteSize::from(NumberPrefix::Prefixed(Prefix::Zebi, 10_f32)))).is_err());
    }
}
