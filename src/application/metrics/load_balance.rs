use prometheus_client::{
    encoding::text::Encode,
    metrics::family::Family,
    registry::Registry,
};

use crate::{
    application::metrics::{Collector, Gauge},
    domain::load_balance::{
        LoadBalanceInterface,
        LoadBalancePing,
        LoadBalanceStatus,
    },
    service::load_balance::LoadBalanceGroupResult,
};

pub struct LoadBalanceHealthLabelBuilder {
    interface_name: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Encode)]
pub struct LoadBalanceHealthLabel {
    group_name: String,
    interface_name: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Encode)]
pub struct LoadBalancePingLabel {
    group_name: String,
    interface_name: String,
    gateway: String,
}

impl From<LoadBalanceInterface> for LoadBalanceHealthLabelBuilder {
    fn from(i: LoadBalanceInterface) -> Self {
        let interface_name = i.interface;
        Self {
            interface_name,
        }
    }
}

impl LoadBalanceHealthLabelBuilder {
    pub fn with(self, group_name: &str) -> LoadBalanceHealthLabel {
        let group_name = group_name.to_string();
        let interface_name = self.interface_name;
        LoadBalanceHealthLabel {
            group_name,
            interface_name,
        }
    }
}

impl LoadBalanceHealthLabel {
    pub fn with(self, gateway: &str) -> LoadBalancePingLabel {
        let group_name = self.group_name;
        let interface_name = self.interface_name;
        let gateway = gateway.to_string();
        LoadBalancePingLabel {
            group_name,
            interface_name,
            gateway,
        }
    }
}

impl Collector for LoadBalanceGroupResult {
    fn collect(self, registry: &mut Registry) {
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

        for load_balance in self {
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
    }
}
