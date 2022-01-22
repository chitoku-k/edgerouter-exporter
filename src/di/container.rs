use std::sync::Arc;

use crate::{
    application::{metrics::MetricsController, server::Engine},
    infrastructure::{
        cmd::{
            parser::{
                bgp::BGPParser,
                ddns::DdnsParser,
                load_balance::LoadBalanceParser,
                pppoe::PPPoEParser,
                version::VersionParser,
            },
            runner::{
                bgp::BGPRunner,
                ddns::DdnsRunner,
                load_balance::LoadBalanceRunner,
                pppoe::PPPoERunner,
                version::VersionRunner,
            },
        },
        config::env::Config,
    },
};

pub struct Application;

impl Application {
    pub async fn start(config: &Config) -> anyhow::Result<()> {
        let engine = Engine::new(
            config.port,
            config.tls_cert.clone(),
            config.tls_key.clone(),
            MetricsController::new(
                BGPRunner::new(&config.vtysh_command, BGPParser),
                DdnsRunner::new(&config.op_ddns_command, DdnsParser),
                LoadBalanceRunner::new(&config.op_command, LoadBalanceParser),
                PPPoERunner::new(&config.op_command, PPPoEParser),
                VersionRunner::new(&config.op_command, VersionParser),
            ),
        );
        Ok(Arc::new(engine).start().await)
    }
}
