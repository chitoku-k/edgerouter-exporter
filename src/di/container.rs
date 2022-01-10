use crate::infrastructure::{
    cmd::{
        runner::{
            bgp::BGPRunner,
            ddns::DdnsRunner,
            load_balance::LoadBalanceRunner,
            pppoe::PPPoERunner,
            version::VersionRunner,
        },
        parser::{
            bgp::BGPParser,
            ddns::DdnsParser,
            load_balance::LoadBalanceParser,
            pppoe::PPPoEParser,
            version::VersionParser,
        },
    },
    config::env::Config,
};

pub struct Application<'a> {
    pub bgp_runner: BGPRunner<'a, BGPParser>,
    pub ddns_runner: DdnsRunner<'a, DdnsParser>,
    pub load_balance_runner: LoadBalanceRunner<'a, LoadBalanceParser>,
    pub pppoe_runner: PPPoERunner<'a, PPPoEParser>,
    pub version_runner: VersionRunner<'a, VersionParser>,
}

impl Application<'_> {
    pub fn new(config: &Config) -> Application {
        Application {
            bgp_runner: BGPRunner::new(&config.vtysh_command, BGPParser),
            ddns_runner: DdnsRunner::new(&config.op_ddns_command, DdnsParser),
            load_balance_runner: LoadBalanceRunner::new(&config.op_command, LoadBalanceParser),
            pppoe_runner: PPPoERunner::new(&config.op_command, PPPoEParser),
            version_runner: VersionRunner::new(&config.op_command, VersionParser),
        }
    }
}
