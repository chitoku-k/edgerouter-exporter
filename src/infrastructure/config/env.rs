use anyhow::{anyhow, Context};
use derive_more::{Deref, From};
use envy::Error;
use serde::Deserialize;

#[derive(Clone, Debug, Deref, Deserialize, From, PartialEq)]
pub struct ViciPath(String);

#[derive(Clone, Debug, Deref, Deserialize, From, PartialEq)]
pub struct IpCommand(String);

#[derive(Clone, Debug, Deref, Deserialize, From, PartialEq)]
pub struct OpCommand(String);

#[derive(Clone, Debug, Deref, Deserialize, From, PartialEq)]
pub struct OpDdnsCommand(String);

#[derive(Clone, Debug, Deref, Deserialize, From, PartialEq)]
pub struct VtyshCommand(String);

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub port: u16,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,

    #[serde(default = "default_vici_path")]
    pub vici_path: ViciPath,

    #[serde(default = "default_ip_command")]
    pub ip_command: IpCommand,

    #[serde(default = "default_op_command")]
    pub op_command: OpCommand,

    #[serde(default = "default_op_ddns_command")]
    pub op_ddns_command: OpDdnsCommand,

    #[serde(default = "default_vtysh_command")]
    pub vtysh_command: VtyshCommand,
}

pub fn get() -> anyhow::Result<Config> {
    envy::from_env()
        .map_err(|e| match e {
            Error::MissingValue(field) => anyhow!("missing value for {}", field.to_ascii_uppercase()),
            Error::Custom(e) => anyhow!(e),
        })
        .context("error reading environment variables")
}

fn default_vici_path() -> ViciPath {
    ViciPath("/run/charon.vici".to_string())
}

fn default_ip_command() -> IpCommand {
    IpCommand("/bin/ip".to_string())
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
