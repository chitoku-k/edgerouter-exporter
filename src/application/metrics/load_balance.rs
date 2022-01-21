use prometheus_client::encoding::text::Encode;

use crate::domain::load_balance::LoadBalanceInterface;

pub struct LoadBalanceHealthLabelBuilder {
    interface_name: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Encode)]
pub struct LoadBalanceHealthLabel {
    group_name: String,
    interface_name: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Encode)]
pub struct LoadBalancePingLabel {
    group_name: String,
    interface_name: String,
    gateway: String,
}

impl From<LoadBalanceInterface> for LoadBalanceHealthLabelBuilder {
    fn from(i: LoadBalanceInterface) -> Self {
        let interface_name = i.interface;
        Self {
            interface_name,
        }
    }
}

impl LoadBalanceHealthLabelBuilder {
    pub fn with(self, group_name: &str) -> LoadBalanceHealthLabel {
        let group_name = group_name.to_string();
        let interface_name = self.interface_name;
        LoadBalanceHealthLabel {
            group_name,
            interface_name,
        }
    }
}

impl LoadBalanceHealthLabel {
    pub fn with(self, gateway: &str) -> LoadBalancePingLabel {
        let group_name = self.group_name;
        let interface_name = self.interface_name;
        let gateway = gateway.to_string();
        LoadBalancePingLabel {
            group_name,
            interface_name,
            gateway,
        }
    }
}
