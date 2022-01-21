use chrono::NaiveDateTime;

#[derive(Clone, Debug, PartialEq)]
pub struct LoadBalanceGroup {
    pub name: String,
    pub interfaces: Vec<LoadBalanceInterface>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadBalanceInterface {
    pub interface: String,
    pub status: LoadBalanceStatus,
    pub failover_only_mode: bool,
    pub pings: u64,
    pub fails: u64,
    pub run_fails: (u64, u64),
    pub route_drops: u64,
    pub ping: LoadBalancePing,
    pub last_route_drop: Option<NaiveDateTime>,
    pub last_route_recover: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LoadBalanceStatus {
    Ok,
    Running,
    WaitOnRecovery(u64, u64),
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum LoadBalancePing {
    Reachable(String),
    Down(String),
    Unknown(String, String),
}
