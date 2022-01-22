use async_trait::async_trait;
use prometheus_client::{
    encoding::text::encode,
    registry::Registry,
};
use tokio::try_join;
use warp::{Reply, reply, hyper::StatusCode};

use crate::{
    application::server::Controller,
    service::{
        bgp::BGPStatusResult,
        ddns::DdnsStatusResult,
        load_balance::LoadBalanceGroupResult,
        pppoe::PPPoEClientSessionResult,
        version::VersionResult,
        Runner,
    },
};

mod bgp;
mod ddns;
mod load_balance;
mod pppoe;
mod version;

pub trait Collector {
    fn collect(self, registry: &mut Registry);
}

#[derive(Clone)]
pub struct MetricsController<BGPRunner, DdnsRunner, LoadBalanceRunner, PPPoERunner, VersionRunner>
where
    BGPRunner: Runner<Item = (BGPStatusResult, BGPStatusResult)> + Send + Sync + Clone,
    DdnsRunner: Runner<Item = DdnsStatusResult> + Send + Sync + Clone,
    LoadBalanceRunner: Runner<Item = LoadBalanceGroupResult> + Send + Sync + Clone,
    PPPoERunner: Runner<Item = PPPoEClientSessionResult> + Send + Sync + Clone,
    VersionRunner: Runner<Item = VersionResult> + Send + Sync + Clone,
{
    bgp_runner: BGPRunner,
    ddns_runner: DdnsRunner,
    load_balance_runner: LoadBalanceRunner,
    pppoe_runner: PPPoERunner,
    version_runner: VersionRunner,
}

impl<BGPRunner, DdnsRunner, LoadBalanceRunner, PPPoERunner, VersionRunner> MetricsController<BGPRunner, DdnsRunner, LoadBalanceRunner, PPPoERunner, VersionRunner>
where
    BGPRunner: Runner<Item = (BGPStatusResult, BGPStatusResult)> + Send + Sync + Clone + 'static,
    DdnsRunner: Runner<Item = DdnsStatusResult> + Send + Sync + Clone + 'static,
    LoadBalanceRunner: Runner<Item = LoadBalanceGroupResult> + Send + Sync + Clone + 'static,
    PPPoERunner: Runner<Item = PPPoEClientSessionResult> + Send + Sync + Clone + 'static,
    VersionRunner: Runner<Item = VersionResult> + Send + Sync + Clone + 'static,
{
    pub fn new(
        bgp_runner: BGPRunner,
        ddns_runner: DdnsRunner,
        load_balance_runner: LoadBalanceRunner,
        pppoe_runner: PPPoERunner,
        version_runner: VersionRunner,
    ) -> Self {
        Self {
            bgp_runner,
            ddns_runner,
            load_balance_runner,
            pppoe_runner,
            version_runner,
        }
    }

    async fn collect(&self, mut registry: Registry) -> anyhow::Result<String> {
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

        bgp.collect(&mut registry);
        ddns.collect(&mut registry);
        load_balance_groups.collect(&mut registry);
        pppoe_client_sessions.collect(&mut registry);
        version.collect(&mut registry);

        let mut buf = vec![];
        encode(&mut buf, &registry)?;

        Ok(String::from_utf8(buf)?)
    }
}

#[async_trait]
impl<BGPRunner, DdnsRunner, LoadBalanceRunner, PPPoERunner, VersionRunner> Controller for MetricsController<BGPRunner, DdnsRunner, LoadBalanceRunner, PPPoERunner, VersionRunner>
where
    BGPRunner: Runner<Item = (BGPStatusResult, BGPStatusResult)> + Send + Sync + Clone + 'static,
    DdnsRunner: Runner<Item = DdnsStatusResult> + Send + Sync + Clone + 'static,
    LoadBalanceRunner: Runner<Item = LoadBalanceGroupResult> + Send + Sync + Clone + 'static,
    PPPoERunner: Runner<Item = PPPoEClientSessionResult> + Send + Sync + Clone + 'static,
    VersionRunner: Runner<Item = VersionResult> + Send + Sync + Clone + 'static,
{
    async fn handle(&self) -> Box<dyn Reply> {
        let registry = Registry::default();

        match self.collect(registry).await {
            Ok(r) => {
                Box::new(reply::with_status(r, StatusCode::OK))
            },
            Err(e) => {
                eprintln!("Internal Server Error: {:?}", e);
                Box::new(reply::with_status("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR))
            },
        }
    }
}
