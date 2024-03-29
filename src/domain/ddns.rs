use std::net::IpAddr;

use chrono::NaiveDateTime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DdnsStatus {
    pub interface: String,
    pub ip_address: Option<IpAddr>,
    pub host_name: Option<String>,
    pub last_update: Option<NaiveDateTime>,
    pub update_status: Option<DdnsUpdateStatus>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DdnsUpdateStatus {
    Abuse,
    BadAgent,
    BadAuth,
    BadSystemParameter,
    Blocked,
    DNSError,
    Failed,
    Good,
    Illegal,
    NoAccess,
    NoChange,
    NoConnect,
    NoError,
    NoFQDN,
    NoHost,
    NoService,
    NotActive,
    NotDonator,
    NotDynamicHost,
    NotYours,
    NumHost,
    TooSoon,
    Unauthenticated,
    Unknown(String),
}
