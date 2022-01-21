use prometheus_client::encoding::text::Encode;

use crate::domain::pppoe::PPPoEClientSession;

#[derive(Clone, Hash, PartialEq, Eq, Encode)]
pub struct PPPoEClientSessionLabel {
    user: String,
    protocol: String,
    interface_name: String,
    ip_address: String,
}

impl From<PPPoEClientSession> for PPPoEClientSessionLabel {
    fn from(s: PPPoEClientSession) -> Self {
        let user = s.user;
        let protocol = s.protocol;
        let interface_name = s.interface;
        let ip_address = s.remote_ip.to_string();
        Self {
            user,
            protocol,
            interface_name,
            ip_address,
        }
    }
}
