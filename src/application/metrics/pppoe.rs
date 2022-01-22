use prometheus_client::{
    encoding::text::Encode,
    registry::Registry,
    metrics::{family::Family, gauge::Gauge},
};

use crate::{
    application::metrics::Collector,
    domain::pppoe::PPPoEClientSession,
    service::pppoe::PPPoEClientSessionResult,
};

#[derive(Clone, Hash, PartialEq, Eq, Encode)]
pub struct PPPoEClientSessionLabel {
    user: String,
    protocol: String,
    interface_name: String,
    ip_address: String,
}

impl From<PPPoEClientSession> for PPPoEClientSessionLabel {
    fn from(s: PPPoEClientSession) -> Self {
        let user = s.user;
        let protocol = s.protocol;
        let interface_name = s.interface;
        let ip_address = s.remote_ip.to_string();
        Self {
            user,
            protocol,
            interface_name,
            ip_address,
        }
    }
}

impl Collector for PPPoEClientSessionResult {
    fn collect(self, registry: &mut Registry) {
        let pppoe_client_session_seconds_total = Family::<PPPoEClientSessionLabel, Gauge>::default();
        registry.register(
            "edgerouter_pppoe_client_session_seconds_total",
            "Total seconds for PPPoE client session",
            Box::new(pppoe_client_session_seconds_total.clone()),
        );

        let pppoe_client_session_transmit_packets_total = Family::<PPPoEClientSessionLabel, Gauge>::default();
        registry.register(
            "edgerouter_pppoe_client_session_transmit_packets_total",
            "Total transmit packets for PPPoE client session",
            Box::new(pppoe_client_session_transmit_packets_total.clone()),
        );

        let pppoe_client_session_receive_packets_total = Family::<PPPoEClientSessionLabel, Gauge>::default();
        registry.register(
            "edgerouter_pppoe_client_session_receive_packets_total",
            "Total receive packets for PPPoE client session",
            Box::new(pppoe_client_session_receive_packets_total.clone()),
        );

        let pppoe_client_session_transmit_bytes_total = Family::<PPPoEClientSessionLabel, Gauge>::default();
        registry.register(
            "edgerouter_pppoe_client_session_transmit_bytes_total",
            "Total transmit bytes for PPPoE client session",
            Box::new(pppoe_client_session_transmit_bytes_total.clone()),
        );

        let pppoe_client_session_receive_bytes_total = Family::<PPPoEClientSessionLabel, Gauge>::default();
        registry.register(
            "edgerouter_pppoe_client_session_receive_bytes_total",
            "Total receive bytes for PPPoE client session",
            Box::new(pppoe_client_session_receive_bytes_total.clone()),
        );

        for session in self {
            let (
                seconds,
                transmit_packets,
                receive_packets,
                transmit_bytes,
                receive_bytes,
            ) = (
                session.time.as_secs(),
                session.transmit_packets.clone().into(),
                session.receive_packets.clone().into(),
                session.transmit_bytes.clone().into(),
                session.receive_bytes.clone().into(),
            );
            let labels = session.into();

            pppoe_client_session_seconds_total
                .get_or_create(&labels)
                .set(seconds);

            pppoe_client_session_transmit_packets_total
                .get_or_create(&labels)
                .set(transmit_packets);

            pppoe_client_session_receive_packets_total
                .get_or_create(&labels)
                .set(receive_packets);

            pppoe_client_session_transmit_bytes_total
                .get_or_create(&labels)
                .set(transmit_bytes);

            pppoe_client_session_receive_bytes_total
                .get_or_create(&labels)
                .set(receive_bytes);
        }
    }
}
