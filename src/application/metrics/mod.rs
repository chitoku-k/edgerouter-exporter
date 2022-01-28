use std::sync::Arc;

use async_trait::async_trait;
use axum::{extract::Extension, http::StatusCode, response::IntoResponse};
use derive_more::Constructor;
use prometheus_client::{encoding::text::encode, registry::Registry};
use tokio::try_join;

use crate::{
    application::server::Controller,
    service::{
        bgp::BGPStatusResult,
        ddns::DdnsStatusResult,
        interface::InterfaceResult,
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

pub trait Collector<T = ()> {
    fn collect(self, registry: &mut Registry, args: T);
}

#[derive(Clone, Constructor)]
pub struct MetricsHandler<BGPRunner, DdnsRunner, InterfaceRunner, LoadBalanceRunner, PPPoERunner, VersionRunner>
where
    BGPRunner: Runner<Item = (BGPStatusResult, BGPStatusResult)> + Send + Sync + Clone,
    DdnsRunner: Runner<Item = DdnsStatusResult> + Send + Sync + Clone,
    InterfaceRunner: Runner<Item = InterfaceResult> + Send + Sync + Clone,
    LoadBalanceRunner: Runner<Item = LoadBalanceGroupResult> + Send + Sync + Clone,
    PPPoERunner: Runner<Item = PPPoEClientSessionResult> + Send + Sync + Clone,
    VersionRunner: Runner<Item = VersionResult> + Send + Sync + Clone,
{
    bgp_runner: BGPRunner,
    ddns_runner: DdnsRunner,
    interface_runner: InterfaceRunner,
    load_balance_runner: LoadBalanceRunner,
    pppoe_runner: PPPoERunner,
    version_runner: VersionRunner,
}

#[async_trait]
impl<BGPRunner, DdnsRunner, InterfaceRunner, LoadBalanceRunner, PPPoERunner, VersionRunner> Controller<String>
    for MetricsHandler<BGPRunner, DdnsRunner, InterfaceRunner, LoadBalanceRunner, PPPoERunner, VersionRunner>
where
    BGPRunner: Runner<Item = (BGPStatusResult, BGPStatusResult)> + Send + Sync + Clone + 'static,
    DdnsRunner: Runner<Item = DdnsStatusResult> + Send + Sync + Clone + 'static,
    InterfaceRunner: Runner<Item = InterfaceResult> + Send + Sync + Clone + 'static,
    LoadBalanceRunner: Runner<Item = LoadBalanceGroupResult> + Send + Sync + Clone + 'static,
    PPPoERunner: Runner<Item = PPPoEClientSessionResult> + Send + Sync + Clone + 'static,
    VersionRunner: Runner<Item = VersionResult> + Send + Sync + Clone + 'static,
{
    async fn handle(&self) -> anyhow::Result<String> {
        let mut registry = Registry::default();
        let (
            bgp,
            ddns,
            interfaces,
            load_balance_groups,
            pppoe_client_sessions,
            version,
        ) = try_join!(
            self.bgp_runner.run(),
            self.ddns_runner.run(),
            self.interface_runner.run(),
            self.load_balance_runner.run(),
            self.pppoe_runner.run(),
            self.version_runner.run(),
        )?;

        bgp.collect(&mut registry, ());
        ddns.collect(&mut registry, ());
        load_balance_groups.collect(&mut registry, ());
        pppoe_client_sessions.collect(&mut registry, &interfaces);
        version.collect(&mut registry, ());

        let mut buf = vec![];
        encode(&mut buf, &registry)?;

        Ok(String::from_utf8(buf)?)
    }
}

pub async fn handle<T>(Extension(controller): Extension<Arc<T>>) -> impl IntoResponse
where
    T: Controller<String>,
{
    match controller.handle().await {
        Ok(s) => {
            (StatusCode::OK, s)
        },
        Err(e) => {
            log::error!("failed to collect metrics: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, String::new())
        },
    }
}
