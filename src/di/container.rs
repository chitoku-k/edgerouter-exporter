use log::LevelFilter;

use crate::{
    application::{metrics::MetricsHandler, server::Engine},
    infrastructure::{
        client::runner::ipsec::IPsecRunner,
        cmd::{
            parser::{
                bgp::BGPParser,
                ddns::DdnsParser,
                interface::InterfaceParser,
                load_balance::{LoadBalanceStatusParser, LoadBalanceWatchdogParser},
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
            .format_indent(None)
            .filter(None, LevelFilter::Info)
            .parse_env("LOG_LEVEL")
            .init();

        let config = env::get();
        let engine = Engine::new(
            config.port,
            config.tls_cert,
            config.tls_key,
            MetricsHandler::new(
                BGPRunner::new(config.vtysh_command, CommandExecutor, BGPParser),
                DdnsRunner::new(config.op_ddns_command, CommandExecutor, DdnsParser),
                IPsecRunner::new(config.vici_path),
                LoadBalanceRunner::new(config.op_command.clone(), CommandExecutor, LoadBalanceStatusParser, LoadBalanceWatchdogParser),
                PPPoERunner::new(config.op_command.clone(), config.ip_command, CommandExecutor, PPPoEParser, InterfaceParser),
                VersionRunner::new(config.op_command, CommandExecutor, VersionParser),
            ),
        );

        engine.start().await
    }
}
