use prometheus_client::{
    encoding::text::Encode,
    metrics::family::Family,
    registry::Registry,
};

use crate::{
    application::metrics::{Collector, Gauge},
    domain::ipsec::{ChildSAState, SA, SAState},
    service::ipsec::IPsecResult,
};

#[derive(Clone, Hash, PartialEq, Eq, Encode)]
pub struct IPsecTunnelLabel {
    tunnel: String,
}

impl From<SA> for IPsecTunnelLabel {
    fn from(sa: SA) -> Self {
        let tunnel = sa.child_sas.into_values()
            .map(|c| c.name)
            .next()
            .unwrap_or_default();

        Self {
            tunnel,
        }
    }
}

impl Collector for IPsecResult {
    fn collect(self, registry: &mut Registry) {
        let ipsec_up = Family::<IPsecTunnelLabel, Gauge>::default();
        registry.register(
            "ipsec_up",
            "Result of IPsec metrics scrape",
            Box::new(ipsec_up.clone()),
        );

        let ipsec_status = Family::<IPsecTunnelLabel, Gauge>::default();
        registry.register(
            "ipsec_status",
            "Status of IPsec tunnel",
            Box::new(ipsec_status.clone()),
        );

        let ipsec_in_bytes = Family::<IPsecTunnelLabel, Gauge>::default();
        registry.register(
            "ipsec_in_bytes",
            "Total receive bytes for IPsec tunnel",
            Box::new(ipsec_in_bytes.clone()),
        );

        let ipsec_out_bytes = Family::<IPsecTunnelLabel, Gauge>::default();
        registry.register(
            "ipsec_out_bytes",
            "Total transmit bytes for IPsec tunnel",
            Box::new(ipsec_out_bytes.clone()),
        );

        let ipsec_in_packets = Family::<IPsecTunnelLabel, Gauge>::default();
        registry.register(
            "ipsec_in_packets",
            "Total receive packets for IPsec tunnel",
            Box::new(ipsec_in_packets.clone()),
        );

        let ipsec_out_packets = Family::<IPsecTunnelLabel, Gauge>::default();
        registry.register(
            "ipsec_out_packets",
            "Total transmit packets for IPsec tunnel",
            Box::new(ipsec_out_packets.clone()),
        );

        for sa in self.into_values() {
            let child_sa = sa.child_sas.values().next();
            let (
                in_bytes,
                out_bytes,
                in_packets,
                out_packets,
            ) = (
                child_sa.map(|c| c.bytes_in).unwrap_or_default(),
                child_sa.map(|c| c.bytes_out).unwrap_or_default(),
                child_sa.map(|c| c.packets_in).unwrap_or_default(),
                child_sa.map(|c| c.packets_out).unwrap_or_default(),
            );
            let status = match (&sa.state, child_sa.map(|c| &c.state)) {
                (SAState::Unknown, _) | (_, Some(ChildSAState::Unknown) | None) => 3,
                (SAState::Established, Some(ChildSAState::Installed | ChildSAState::Rekeying | ChildSAState::Rekeyed)) => 0,
                (SAState::Established, Some(_)) => 1,
                _ => 2,
            };
            let labels = sa.into();

            ipsec_up
                .get_or_create(&labels)
                .set(1);

            ipsec_status
                .get_or_create(&labels)
                .set(status);

            ipsec_in_bytes
                .get_or_create(&labels)
                .set(in_bytes);

            ipsec_out_bytes
                .get_or_create(&labels)
                .set(out_bytes);

            ipsec_in_packets
                .get_or_create(&labels)
                .set(in_packets);

            ipsec_out_packets
                .get_or_create(&labels)
                .set(out_packets);
        }
    }
}
