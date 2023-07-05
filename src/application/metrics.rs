use std::sync::Arc;

use async_trait::async_trait;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use derive_more::Constructor;
use prometheus_client::{encoding::text::encode, metrics::gauge, registry::Registry};
use tokio::try_join;

use crate::{
    application::server::Controller,
    service::{
        bgp::BGPStatusResult,
        ddns::DdnsStatusResult,
        ipsec::IPsecResult,
        load_balance::LoadBalanceStatusResult,
        pppoe::PPPoEClientSessionResult,
        version::VersionResult,
        Runner,
    },
};

mod atomic;
mod bgp;
mod ddns;
mod ipsec;
mod load_balance;
mod pppoe;
mod version;

pub type Gauge<T = i64, U = atomic::AtomicI64> = gauge::Gauge<T, U>;

pub trait Collector {
    fn collect(self, registry: &mut Registry);
}

#[derive(Constructor)]
pub struct MetricsHandler<BGPRunner, DdnsRunner, IPsecRunner, LoadBalanceRunner, PPPoERunner, VersionRunner> {
    bgp_runner: BGPRunner,
    ddns_runner: DdnsRunner,
    ipsec_runner: IPsecRunner,
    load_balance_runner: LoadBalanceRunner,
    pppoe_runner: PPPoERunner,
    version_runner: VersionRunner,
}

#[async_trait]
impl<BGPRunner, DdnsRunner, IPsecRunner, LoadBalanceRunner, PPPoERunner, VersionRunner> Controller<String>
    for MetricsHandler<BGPRunner, DdnsRunner, IPsecRunner, LoadBalanceRunner, PPPoERunner, VersionRunner>
where
    BGPRunner: Runner<Item = (BGPStatusResult, BGPStatusResult)> + Send + Sync + 'static,
    DdnsRunner: Runner<Item = DdnsStatusResult> + Send + Sync + 'static,
    IPsecRunner: Runner<Item = IPsecResult> + Send + Sync + 'static,
    LoadBalanceRunner: Runner<Item = LoadBalanceStatusResult> + Send + Sync + 'static,
    PPPoERunner: Runner<Item = PPPoEClientSessionResult> + Send + Sync + 'static,
    VersionRunner: Runner<Item = VersionResult> + Send + Sync + 'static,
{
    async fn handle(&self) -> anyhow::Result<String> {
        let mut registry = Registry::default();
        let (
            bgp,
            ddns,
            ipsec_sas,
            load_balance_groups,
            pppoe_client_sessions,
            version,
        ) = try_join!(
            self.bgp_runner.run(),
            self.ddns_runner.run(),
            self.ipsec_runner.run(),
            self.load_balance_runner.run(),
            self.pppoe_runner.run(),
            self.version_runner.run(),
        )?;

        bgp.collect(&mut registry);
        ddns.collect(&mut registry);
        ipsec_sas.collect(&mut registry);
        load_balance_groups.collect(&mut registry);
        pppoe_client_sessions.collect(&mut registry);
        version.collect(&mut registry);

        let mut buf = String::new();
        encode(&mut buf, &registry)?;

        Ok(buf)
    }
}

pub async fn handle<T>(State(controller): State<Arc<T>>) -> impl IntoResponse
where
    T: Controller<String>,
{
    match controller.handle().await {
        Ok(s) => {
            (StatusCode::OK, s)
        },
        Err(e) => {
            log::error!("failed to collect metrics\nError: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, String::new())
        },
    }
}
