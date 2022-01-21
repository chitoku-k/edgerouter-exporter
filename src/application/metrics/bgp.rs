use prometheus_client::encoding::text::Encode;

use crate::domain::bgp::BGPNeighbor;

#[derive(Clone, Hash, PartialEq, Eq, Encode)]
pub struct BGPNeighborLabel {
    neighbor: String,
    r#as: String,
    table_version: String,
}

impl From<BGPNeighbor> for BGPNeighborLabel {
    fn from(n: BGPNeighbor) -> Self {
        let neighbor = n.neighbor.to_string();
        let r#as = n.remote_as.to_string();
        let table_version = n.table_version.to_string();
        Self {
            neighbor,
            r#as,
            table_version,
        }
    }
}
