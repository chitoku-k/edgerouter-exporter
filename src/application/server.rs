use std::{net::Ipv6Addr, convert::Infallible, sync::Arc};

use prometheus_client::{
    encoding::text::encode,
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};
use tokio::try_join;
use warp::{hyper::StatusCode, reply, Filter};

use crate::{
    application::metrics::{
        bgp::BGPNeighborLabel,
        ddns::DdnsStatusLabel,
        load_balance::{LoadBalanceHealthLabel, LoadBalanceHealthLabelBuilder, LoadBalancePingLabel},
        pppoe::PPPoEClientSessionLabel,
        version::VersionLabel,
    },
    domain::{
        bgp::BGPIterator,
        ddns::DdnsUpdateStatus,
        load_balance::{LoadBalancePing, LoadBalanceStatus},
    },
    service::{
        bgp::BGPStatusResult,
        ddns::DdnsStatusResult,
        load_balance::LoadBalanceGroupResult,
        pppoe::PPPoEClientSessionResult,
        version::VersionResult,
        Runner,
    },
};

#[derive(Clone)]
pub struct Engine<BGPRunner, DdnsRunner, LoadBalanceRunner, PPPoERunner, VersionRunner>
where
    BGPRunner: Runner<Item = (BGPStatusResult, BGPStatusResult)> + Send + Sync + Clone,
    DdnsRunner: Runner<Item = DdnsStatusResult> + Send + Sync + Clone,
    LoadBalanceRunner: Runner<Item = LoadBalanceGroupResult> + Send + Sync + Clone,
    PPPoERunner: Runner<Item = PPPoEClientSessionResult> + Send + Sync + Clone,
    VersionRunner: Runner<Item = VersionResult> + Send + Sync + Clone,
{
    port: u16,
    tls: Option<(String, String)>,

    bgp_runner: BGPRunner,
    ddns_runner: DdnsRunner,
    load_balance_runner: LoadBalanceRunner,
    pppoe_runner: PPPoERunner,
    version_runner: VersionRunner,
}

impl<BGPRunner, DdnsRunner, LoadBalanceRunner, PPPoERunner, VersionRunner> Engine<BGPRunner, DdnsRunner, LoadBalanceRunner, PPPoERunner, VersionRunner>
where
    BGPRunner: Runner<Item = (BGPStatusResult, BGPStatusResult)> + Send + Sync + Clone + 'static,
    DdnsRunner: Runner<Item = DdnsStatusResult> + Send + Sync + Clone + 'static,
    LoadBalanceRunner: Runner<Item = LoadBalanceGroupResult> + Send + Sync + Clone + 'static,
    PPPoERunner: Runner<Item = PPPoEClientSessionResult> + Send + Sync + Clone + 'static,
    VersionRunner: Runner<Item = VersionResult> + Send + Sync + Clone + 'static,
{
    pub fn new(
        port: u16,
        tls_cert: Option<String>,
        tls_key: Option<String>,
        bgp_runner: BGPRunner,
        ddns_runner: DdnsRunner,
        load_balance_runner: LoadBalanceRunner,
        pppoe_runner: PPPoERunner,
        version_runner: VersionRunner,
    ) -> anyhow::Result<Self> {
        let tls = Option::zip(tls_cert, tls_key);
        Ok(Self {
            port,
            tls,
            bgp_runner,
            ddns_runner,
            load_balance_runner,
            pppoe_runner,
            version_runner,
        })
    }

    async fn metrics_handler(&self) -> anyhow::Result<String> {
        let mut registry = <Registry>::default();
        let (
            bgp,
            ddns,
            load_balance_groups,
            pppoe_client_sessions,
            version,
        ) = try_join!(
            self.bgp_runner.run(),
            self.ddns_runner.run(),
            self.load_balance_runner.run(),
            self.pppoe_runner.run(),
            self.version_runner.run(),
        )?;

        let info = Family::<VersionLabel, Gauge>::default();
        registry.register(
            "edgerouter_info",
            "Version info",
            Box::new(info.clone()),
        );

        info.get_or_create(&version.into()).set(1);

        let bgp_msg_rcv = Family::<BGPNeighborLabel, Gauge>::default();
        registry.register(
            "edgerouter_bgp_message_received_total",
            "Total number of BGP messages received",
            Box::new(bgp_msg_rcv.clone()),
        );

        let bgp_msg_sen = Family::<BGPNeighborLabel, Gauge>::default();
        registry.register(
            "edgerouter_bgp_message_sent_total",
            "Total number of BGP messages sent",
            Box::new(bgp_msg_sen.clone()),
        );

        let bgp_in_q = Family::<BGPNeighborLabel, Gauge>::default();
        registry.register(
            "edgerouter_bgp_message_in_queue",
            "Number of BGP messages in incoming queue",
            Box::new(bgp_in_q.clone()),
        );

        let bgp_out_q = Family::<BGPNeighborLabel, Gauge>::default();
        registry.register(
            "edgerouter_bgp_message_out_queue",
            "Number of BGP messages in outgoing queue",
            Box::new(bgp_out_q.clone()),
        );

        let bgp_session_seconds_total = Family::<BGPNeighborLabel, Gauge>::default();
        registry.register(
            "edgerouter_bgp_session_seconds_total",
            "Total seconds for established BGP session",
            Box::new(bgp_session_seconds_total.clone()),
        );

        let bgp_pfx_rcd = Family::<BGPNeighborLabel, Gauge>::default();
        registry.register(
            "edgerouter_bgp_prefix_received_total",
            "Total number of BGP prefixes received",
            Box::new(bgp_pfx_rcd.clone()),
        );

        for neighbor in BGPIterator::from(bgp) {
            let labels = neighbor.clone().into();

            bgp_msg_rcv
                .get_or_create(&labels)
                .set(neighbor.messages_received);

            bgp_msg_sen
                .get_or_create(&labels)
                .set(neighbor.messages_sent);

            bgp_in_q
                .get_or_create(&labels)
                .set(neighbor.in_queue);

            bgp_out_q
                .get_or_create(&labels)
                .set(neighbor.out_queue);

            bgp_session_seconds_total
                .get_or_create(&labels)
                .set(neighbor.uptime.map(|d| d.as_secs()).unwrap_or_default());

            bgp_pfx_rcd
                .get_or_create(&labels)
                .set(neighbor.prefixes_received.unwrap_or_default());
        }

        let ddns_status = Family::<DdnsStatusLabel, Gauge>::default();
        registry.register(
            "edgerouter_dynamic_dns_status",
            "Result of DDNS update",
            Box::new(ddns_status.clone()),
        );

        for status in ddns {
            let value = match status.update_status {
                Some(DdnsUpdateStatus::Good | DdnsUpdateStatus::NoChange | DdnsUpdateStatus::NoError) => 1,
                _ => 0,
            };
            ddns_status.get_or_create(&status.into()).set(value);
        }

        let load_balancer_health = Family::<LoadBalanceHealthLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_health",
            "Result of watchdog",
            Box::new(load_balancer_health.clone()),
        );

        let load_balancer_run_fail_total = Family::<LoadBalanceHealthLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_run_fail_total",
            "Total number of run failures",
            Box::new(load_balancer_run_fail_total.clone()),
        );

        let load_balancer_route_drop_total = Family::<LoadBalanceHealthLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_route_drop_total",
            "Total number of route drops",
            Box::new(load_balancer_route_drop_total.clone()),
        );

        let load_balancer_ping_health = Family::<LoadBalancePingLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_ping_health",
            "Result of ping",
            Box::new(load_balancer_ping_health.clone()),
        );

        let load_balancer_ping_total = Family::<LoadBalancePingLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_ping_total",
            "Total number of pings",
            Box::new(load_balancer_ping_total.clone()),
        );

        let load_balancer_ping_fail_total = Family::<LoadBalancePingLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_ping_fail_total",
            "Total number of ping failures",
            Box::new(load_balancer_ping_fail_total.clone()),
        );

        for load_balance in load_balance_groups {
            for interface in load_balance.interfaces {
                let (
                    health,
                    (run_fails, _),
                    route_drops,
                ) = match interface.status {
                    LoadBalanceStatus::Ok | LoadBalanceStatus::Running => (
                        1,
                        interface.run_fails,
                        interface.route_drops,
                    ),
                    _ => (
                        0,
                        interface.run_fails,
                        interface.route_drops,
                    ),
                };

                let (
                    ping_health,
                    ping_total,
                    ping_fail_total,
                    ping_gateway,
                ) = match interface.ping.clone() {
                    LoadBalancePing::Reachable(gateway) => (
                        1,
                        interface.pings,
                        interface.fails,
                        gateway,
                    ),
                    LoadBalancePing::Down(gateway) | LoadBalancePing::Unknown(_, gateway) => (
                        0,
                        interface.pings,
                        interface.fails,
                        gateway,
                    ),
                };

                let labels = LoadBalanceHealthLabelBuilder::from(interface).with(&load_balance.name);
                load_balancer_health
                    .get_or_create(&labels)
                    .set(health);

                load_balancer_run_fail_total
                    .get_or_create(&labels)
                    .set(run_fails);

                load_balancer_route_drop_total
                    .get_or_create(&labels)
                    .set(route_drops);

                let labels = labels.with(&ping_gateway);
                load_balancer_ping_health
                    .get_or_create(&labels)
                    .set(ping_health);

                load_balancer_ping_total
                    .get_or_create(&labels)
                    .set(ping_total);

                load_balancer_ping_fail_total
                    .get_or_create(&labels)
                    .set(ping_fail_total);
            }
        }

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

        for session in pppoe_client_sessions {
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

        let mut buf = vec![];
        encode(&mut buf, &registry)?;
        Ok(String::from_utf8(buf)?)
    }

    pub async fn start(self: Arc<Self>) {
        let port = self.port;
        let tls = self.tls.clone();

        let metrics = warp::path("metrics").and_then(move || {
            let engine = self.clone();
            async move {
                match engine.metrics_handler().await {
                    Ok(r) => Ok(reply::with_status(r, StatusCode::OK)),
                    Err(e) => {
                        eprintln!("Internal error: {:?}", e);
                        Ok::<_, Infallible>(reply::with_status("Internal Sever Error".to_string(), StatusCode::INTERNAL_SERVER_ERROR))
                    },
                }
            }
        });

        let server = warp::serve(metrics);
        match tls {
            Some((_tls_cert, _tls_key)) => {
                panic!("unsupported");
                // server
                //     .tls()
                //     .cert_path(tls_cert)
                //     .key_path(tls_key)
                //     .run((Ipv6Addr::UNSPECIFIED, port)).await
            },
            None => {
                server
                    .run((Ipv6Addr::UNSPECIFIED, port)).await
            },
        }
    }
}
