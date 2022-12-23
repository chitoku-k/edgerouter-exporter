use prometheus_client::{
    encoding::EncodeLabelSet,
    metrics::family::Family,
    registry::Registry,
};

use crate::{
    application::metrics::{Collector, Gauge},
    domain::ddns::{DdnsStatus, DdnsUpdateStatus},
    service::ddns::DdnsStatusResult,
};

#[derive(Clone, Debug, EncodeLabelSet, Eq, Hash, PartialEq)]
pub struct DdnsStatusLabel {
    interface_name: String,
    ip_address: String,
    hostname: String,
}

impl From<DdnsStatus> for DdnsStatusLabel {
    fn from(s: DdnsStatus) -> Self {
        let interface_name = s.interface;
        let ip_address = s.ip_address.map(|a| a.to_string()).unwrap_or_default();
        let hostname = s.host_name.unwrap_or_default();
        Self {
            interface_name,
            ip_address,
            hostname,
        }
    }
}

impl Collector for DdnsStatusResult {
    fn collect(self, registry: &mut Registry) {
        let ddns_status = Family::<DdnsStatusLabel, Gauge>::default();
        registry.register(
            "edgerouter_dynamic_dns_status",
            "Result of DDNS update",
            ddns_status.clone(),
        );

        for status in self {
            let value = match status.update_status {
                Some(DdnsUpdateStatus::Good | DdnsUpdateStatus::NoChange | DdnsUpdateStatus::NoError) => 1,
                _ => 0,
            };
            ddns_status.get_or_create(&status.into()).set(value);
        }
    }
}
