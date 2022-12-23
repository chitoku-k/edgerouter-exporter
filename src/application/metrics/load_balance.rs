use prometheus_client::{
    encoding::EncodeLabelSet,
    metrics::family::Family,
    registry::Registry,
};

use crate::{
    application::metrics::{atomic, Collector, Gauge},
    domain::load_balance::{
        LoadBalancePing,
        LoadBalanceStatusStatus,
        LoadBalanceWatchdogStatus,
    },
    service::load_balance::LoadBalanceStatusResult,
};

pub struct LoadBalanceHealthLabelBuilder {
    interface_name: String,
}

#[derive(Clone, Debug, EncodeLabelSet, Eq, Hash, PartialEq)]
pub struct LoadBalanceHealthLabel {
    group_name: String,
    interface_name: String,
}

#[derive(Clone, Debug, EncodeLabelSet, Eq, Hash, PartialEq)]
pub struct LoadBalanceFlowLabel {
    group_name: String,
    interface_name: String,
    flow: String,
}

#[derive(Clone, Debug, EncodeLabelSet, Eq, Hash, PartialEq)]
pub struct LoadBalancePingLabel {
    group_name: String,
    interface_name: String,
    gateway: String,
}

impl LoadBalanceHealthLabelBuilder {
    pub fn new(interface_name: String) -> Self {
        Self {
            interface_name,
        }
    }

    pub fn group(self, group_name: String) -> LoadBalanceHealthLabel {
        let interface_name = self.interface_name;
        LoadBalanceHealthLabel {
            group_name,
            interface_name,
        }
    }
}

impl LoadBalanceHealthLabel {
    pub fn flow(self, flow: String) -> LoadBalanceFlowLabel {
        let group_name = self.group_name;
        let interface_name = self.interface_name;
        LoadBalanceFlowLabel {
            group_name,
            interface_name,
            flow,
        }
    }

    pub fn ping(self, gateway: String) -> LoadBalancePingLabel {
        let group_name = self.group_name;
        let interface_name = self.interface_name;
        LoadBalancePingLabel {
            group_name,
            interface_name,
            gateway,
        }
    }
}

impl Collector for LoadBalanceStatusResult {
    fn collect(self, registry: &mut Registry) {
        let load_balancer_status = Family::<LoadBalanceHealthLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_status",
            "Status (0: inactive, 1: active, 2: failover)",
            load_balancer_status.clone(),
        );

        let load_balancer_weight_ratio = Family::<LoadBalanceHealthLabel, Gauge<f64, atomic::AtomicU64>>::default();
        registry.register(
            "edgerouter_load_balancer_weight_ratio",
            "Weight ratio",
            load_balancer_weight_ratio.clone(),
        );

        let load_balancer_flows_total = Family::<LoadBalanceFlowLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_flows_total",
            "Total number of flows",
            load_balancer_flows_total.clone(),
        );

        let load_balancer_health = Family::<LoadBalanceHealthLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_health",
            "Result of watchdog",
            load_balancer_health.clone(),
        );

        let load_balancer_run_fail_total = Family::<LoadBalanceHealthLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_run_fail_total",
            "Total number of run failures",
            load_balancer_run_fail_total.clone(),
        );

        let load_balancer_route_drop_total = Family::<LoadBalanceHealthLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_route_drop_total",
            "Total number of route drops",
            load_balancer_route_drop_total.clone(),
        );

        let load_balancer_ping_health = Family::<LoadBalancePingLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_ping_health",
            "Result of ping",
            load_balancer_ping_health.clone(),
        );

        let load_balancer_ping_total = Family::<LoadBalancePingLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_ping_total",
            "Total number of pings",
            load_balancer_ping_total.clone(),
        );

        let load_balancer_ping_fail_total = Family::<LoadBalancePingLabel, Gauge>::default();
        registry.register(
            "edgerouter_load_balancer_ping_fail_total",
            "Total number of ping failures",
            load_balancer_ping_fail_total.clone(),
        );

        for load_balance in self {
            for interface in load_balance.interfaces {
                let status = match interface.status {
                    LoadBalanceStatusStatus::Inactive | LoadBalanceStatusStatus::Unknown(_) => 0,
                    LoadBalanceStatusStatus::Active => 1,
                    LoadBalanceStatusStatus::Failover => 2,
                };

                let labels = LoadBalanceHealthLabelBuilder::new(interface.interface).group(load_balance.name.clone());
                load_balancer_status
                    .get_or_create(&labels)
                    .set(status);

                load_balancer_weight_ratio
                    .get_or_create(&labels)
                    .set(interface.weight);

                for (flow, value) in interface.flows {
                    let value: u64 = value.into();
                    let labels = labels.clone().flow(flow);
                    load_balancer_flows_total
                        .get_or_create(&labels)
                        .set(value as i64);
                }

                if let Some(watchdog) = interface.watchdog {
                    let (
                        health,
                        (run_fails, _),
                        route_drops,
                    ) = match watchdog.status {
                        LoadBalanceWatchdogStatus::Ok | LoadBalanceWatchdogStatus::Running => (
                            1,
                            watchdog.run_fails,
                            watchdog.route_drops,
                        ),
                        _ => (
                            0,
                            watchdog.run_fails,
                            watchdog.route_drops,
                        ),
                    };

                    let (
                        ping_health,
                        ping_total,
                        ping_fail_total,
                        ping_gateway,
                    ) = match &watchdog.ping {
                        LoadBalancePing::Reachable(gateway) => (
                            1,
                            watchdog.pings,
                            watchdog.fails,
                            gateway.clone(),
                        ),
                        LoadBalancePing::Down(gateway) | LoadBalancePing::Unknown(_, gateway) => (
                            0,
                            watchdog.pings,
                            watchdog.fails,
                            gateway.clone(),
                        ),
                    };

                    let labels = LoadBalanceHealthLabelBuilder::new(watchdog.interface).group(load_balance.name.clone());
                    load_balancer_health
                        .get_or_create(&labels)
                        .set(health);

                    load_balancer_run_fail_total
                        .get_or_create(&labels)
                        .set(run_fails as i64);

                    load_balancer_route_drop_total
                        .get_or_create(&labels)
                        .set(route_drops as i64);

                    let labels = labels.ping(ping_gateway);
                    load_balancer_ping_health
                        .get_or_create(&labels)
                        .set(ping_health);

                    load_balancer_ping_total
                        .get_or_create(&labels)
                        .set(ping_total as i64);

                    load_balancer_ping_fail_total
                        .get_or_create(&labels)
                        .set(ping_fail_total as i64);
                }
            }
        }
    }
}
