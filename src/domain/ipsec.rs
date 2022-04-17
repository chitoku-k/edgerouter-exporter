use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SA {
    pub uniqueid: u32,
    pub version: String,
    pub state: SAState,
    pub local_host: Option<String>,
    pub local_port: Option<u32>,
    pub local_id: String,
    pub remote_host: Option<String>,
    pub remote_port: u32,
    pub remote_id: String,
    pub remote_xauth_id: Option<String>,
    pub remote_eap_id: Option<String>,
    pub encr_alg: Option<String>,
    pub encr_keysize: Option<u32>,
    pub integ_alg: Option<String>,
    pub integ_keysize: Option<u32>,
    pub prf_alg: Option<String>,
    pub dh_group: Option<String>,
    pub established: u64,
    pub rekey_time: Option<u64>,
    pub reauth_time: u64,
    pub child_sas: IndexMap<String, ChildSA>,
}

// See https://github.com/strongswan/strongswan/blob/5.9.5/src/libcharon/sa/ike_sa.h#L287-L365
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SAState {
    Created,
    Connecting,
    Established,
    Passive,
    Rekeying,
    Rekeyed,
    Deleting,
    Destroying,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ChildSA {
    pub name: String,
    pub uniqueid: u32,
    pub reqid: u32,
    pub state: ChildSAState,
    pub mode: String,
    pub protocol: Option<String>,
    pub encr_alg: Option<String>,
    pub encr_keysize: Option<u32>,
    pub integ_alg: Option<String>,
    pub integ_keysize: Option<u32>,
    pub prf_alg: Option<String>,
    pub dh_group: Option<String>,
    pub esn: Option<u32>,
    pub bytes_in: Option<u64>,
    pub packets_in: Option<u64>,
    pub use_in: Option<u64>,
    pub bytes_out: Option<u64>,
    pub packets_out: Option<u64>,
    pub use_out: Option<u64>,
    pub rekey_time: Option<u64>,
    pub life_time: Option<u64>,
    pub install_time: Option<u64>,
    pub local_ts: Vec<String>,
    pub remote_ts: Vec<String>,
}

// See https://github.com/strongswan/strongswan/blob/5.9.5/src/libcharon/sa/child_sa.h#L37-L96
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ChildSAState {
    Created,
    Routed,
    Installing,
    Installed,
    Updating,
    Rekeying,
    Rekeyed,
    Retrying,
    Deleting,
    Destroying,
    #[serde(other)]
    Unknown,
}
