use std::collections::BTreeMap;

use chrono::NaiveDateTime;
use derive_more::{From, FromStr};
use number_prefix::NumberPrefix;

use super::convert_size;

#[derive(Clone, Debug, Eq, From, FromStr, PartialEq)]
pub struct FlowSize(NumberPrefix<u32>);

#[derive(Clone, Debug, PartialEq)]
pub struct LoadBalanceStatus {
    pub name: String,
    pub balance_local: bool,
    pub lock_local_dns: bool,
    pub conntrack_flush: bool,
    pub sticky_bits: u32,
    pub interfaces: Vec<LoadBalanceStatusInterface>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadBalanceStatusInterface {
    pub interface: String,
    pub reachable: bool,
    pub status: LoadBalanceStatusStatus,
    pub gateway: Option<String>,
    pub route_table: u32,
    pub weight: f64,
    pub fo_priority: u32,
    pub flows: BTreeMap<String, FlowSize>,
    pub watchdog: Option<LoadBalanceWatchdogInterface>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoadBalanceStatusStatus {
    Inactive,
    Active,
    Failover,
    Unknown(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadBalanceWatchdog {
    pub name: String,
    pub interfaces: Vec<LoadBalanceWatchdogInterface>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadBalanceWatchdogInterface {
    pub interface: String,
    pub status: LoadBalanceWatchdogStatus,
    pub failover_only_mode: bool,
    pub pings: u64,
    pub fails: u64,
    pub run_fails: (u64, u64),
    pub route_drops: u64,
    pub ping: LoadBalancePing,
    pub last_route_drop: Option<NaiveDateTime>,
    pub last_route_recover: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoadBalanceWatchdogStatus {
    Ok,
    Running,
    WaitOnRecovery(u64, u64),
    Unknown(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoadBalancePing {
    Reachable(String),
    Down(String),
    Unknown(String, String),
}

impl From<FlowSize> for u64 {
    fn from(prefix: FlowSize) -> Self {
        convert_size(prefix.0)
    }
}

#[cfg(test)]
mod tests {
    use std::panic;

    use number_prefix::Prefix;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn convert_flow_size() {
        assert_eq!(u64::from(FlowSize::from(NumberPrefix::Standalone(10_u32))), 10);

        assert_eq!(u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Kilo, 10_u32))), 10 * 1024);
        assert_eq!(u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Mega, 10_u32))), 10 * 1024 * 1024);
        assert_eq!(u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Giga, 10_u32))), 10 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Tera, 10_u32))), 10 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Peta, 10_u32))), 10 * 1024 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Exa, 10_u32))), 10 * 1024 * 1024 * 1024 * 1024 * 1024 * 1024);

        assert_eq!(u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Kibi, 10_u32))), 10 * 1024);
        assert_eq!(u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Mebi, 10_u32))), 10 * 1024 * 1024);
        assert_eq!(u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Gibi, 10_u32))), 10 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Tebi, 10_u32))), 10 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Pebi, 10_u32))), 10 * 1024 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Exbi, 10_u32))), 10 * 1024 * 1024 * 1024 * 1024 * 1024 * 1024);
    }

    #[test]
    fn convert_flow_size_overflow() {
        assert!(panic::catch_unwind(|| u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Yotta, 10_u32)))).is_err());
        assert!(panic::catch_unwind(|| u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Zetta, 10_u32)))).is_err());

        assert!(panic::catch_unwind(|| u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Yobi, 10_u32)))).is_err());
        assert!(panic::catch_unwind(|| u64::from(FlowSize::from(NumberPrefix::Prefixed(Prefix::Zebi, 10_u32)))).is_err());
    }
}
