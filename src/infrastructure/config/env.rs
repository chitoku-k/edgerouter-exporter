use derive_more::Deref;
use envy::Error;
use serde::Deserialize;

#[derive(Clone, Debug, Deref, Deserialize, PartialEq)]
pub struct OpCommand(String);

#[derive(Clone, Debug, Deref, Deserialize, PartialEq)]
pub struct OpDdnsCommand(String);

#[derive(Clone, Debug, Deref, Deserialize, PartialEq)]
pub struct VtyshCommand(String);

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub port: String,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,

    #[serde(default = "default_op_command")]
    pub op_command: OpCommand,

    #[serde(default = "default_op_ddns_command")]
    pub op_ddns_command: OpDdnsCommand,

    #[serde(default = "default_vtysh_command")]
    pub vtysh_command: VtyshCommand,
}

pub fn get() -> anyhow::Result<Config, Error> {
    envy::from_env()
}

fn default_op_command() -> OpCommand {
    OpCommand("/opt/vyatta/bin/vyatta-op-cmd-wrapper".to_string())
}

fn default_op_ddns_command() -> OpDdnsCommand {
    OpDdnsCommand("/opt/vyatta/bin/sudo-users/vyatta-op-dynamic-dns.pl".to_string())
}

fn default_vtysh_command() -> VtyshCommand {
    VtyshCommand("/opt/vyatta/sbin/ubnt_vtysh".to_string())
}
