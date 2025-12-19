use clap::{crate_version, Parser};
use derive_more::{AsRef, Deref, Display, From};

#[derive(AsRef, Clone, Debug, Deref, Display, Eq, From, PartialEq)]
#[as_ref(forward)]
pub struct ViciPath(String);

#[derive(Clone, Debug, Deref, Display, Eq, From, PartialEq)]
pub struct IpCommand(String);

#[derive(Clone, Debug, Deref, Display, Eq, From, PartialEq)]
pub struct OpCommand(String);

#[derive(Clone, Debug, Deref, Display, Eq, From, PartialEq)]
pub struct OpDdnsCommand(String);

#[derive(Clone, Debug, Deref, Display, Eq, From, PartialEq)]
pub struct VtyshCommand(String);

#[derive(Debug, Eq, Parser, PartialEq)]
#[command(version = version())]
pub struct Config {
    /// Log level
    #[arg(long, env, default_value = "info")]
    pub log_level: String,

    /// Port number
    #[arg(long, env)]
    pub port: u16,

    /// Path to TLS certificate (if not specified, exporter is served over HTTP)
    #[arg(long, env, requires = "tls_key")]
    pub tls_cert: Option<String>,

    /// Path to TLS private key (if not specified, exporter is served over HTTP)
    #[arg(long, env, requires = "tls_cert")]
    pub tls_key: Option<String>,

    /// Path to Unix socket for VICI
    #[arg(long, env, default_value_t = default_vici_path())]
    pub vici_path: ViciPath,

    /// Path to ip command
    #[arg(long, env, default_value_t = default_ip_command())]
    pub ip_command: IpCommand,

    /// Path to op command
    #[arg(long, env, default_value_t = default_op_command())]
    pub op_command: OpCommand,

    /// Path to op ddns command
    #[arg(long, env, default_value_t = default_op_ddns_command())]
    pub op_ddns_command: OpDdnsCommand,

    /// Path to vtysh command
    #[arg(long, env, default_value_t = default_vtysh_command())]
    pub vtysh_command: VtyshCommand,
}

pub fn init() -> Config {
    let config = Config::parse();
    config.init();
    config
}

#[cfg(not(feature = "tls"))]
fn version() -> &'static str {
    concat!("v", crate_version!())
}

#[cfg(feature = "tls")]
fn version() -> String {
    format!("{} ({})", concat!("v", crate_version!()), openssl::version::version())
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

impl Config {
    fn init(&self) {
        env_logger::builder()
            .format_target(false)
            .format_timestamp_secs()
            .format_indent(None)
            .parse_filters(&self.log_level)
            .init();
    }
}
