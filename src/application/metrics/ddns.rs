use prometheus_client::encoding::text::Encode;

use crate::domain::ddns::DdnsStatus;

#[derive(Clone, Hash, PartialEq, Eq, Encode)]
pub struct DdnsStatusLabel {
    interface_name: String,
    ip_address: String,
    hostname: String,
}

impl From<DdnsStatus> for DdnsStatusLabel {
    fn from(s: DdnsStatus) -> Self {
        let interface_name = s.interface;
        let ip_address = s.ip_address.map(|a| a.to_string()).unwrap_or_default();
        let hostname = s.host_name;
        Self {
            interface_name,
            ip_address,
            hostname,
        }
    }
}
