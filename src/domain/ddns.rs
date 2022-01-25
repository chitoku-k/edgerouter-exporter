use std::net::IpAddr;

use chrono::NaiveDateTime;

#[derive(Clone, Debug, PartialEq)]
pub struct DdnsStatus {
    pub interface: String,
    pub ip_address: Option<IpAddr>,
    pub host_name: String,
    pub last_update: Option<NaiveDateTime>,
    pub update_status: Option<DdnsUpdateStatus>,
}

#[derive(Clone, Debug, PartialEq)]
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
