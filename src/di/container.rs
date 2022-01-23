use log::LevelFilter;

use crate::{
    application::{metrics::MetricsHandler, server::Engine},
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
                CommandExecutor,
            },
        },
        config::env,
    },
};

pub struct Application;

impl Application {
    pub async fn start() -> anyhow::Result<()> {
        env_logger::builder()
            .format_target(false)
            .format_timestamp_secs()
            .filter(None, LevelFilter::Info)
            .init();

        let config = env::get()?;
        let engine = Engine::new(
            config.port,
            config.tls_cert,
            config.tls_key,
            MetricsHandler::new(
                BGPRunner::new(&config.vtysh_command, CommandExecutor, BGPParser),
                DdnsRunner::new(&config.op_ddns_command, CommandExecutor, DdnsParser),
                LoadBalanceRunner::new(&config.op_command, CommandExecutor, LoadBalanceParser),
                PPPoERunner::new(&config.op_command, CommandExecutor, PPPoEParser),
                VersionRunner::new(&config.op_command, CommandExecutor, VersionParser),
            ),
        );

        engine.start().await
    }
}
