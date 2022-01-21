use prometheus_client::encoding::text::Encode;

use crate::domain::version::Version;

#[derive(Clone, Hash, PartialEq, Eq, Encode)]
pub struct VersionLabel {
    version: String,
    build_id: String,
    model: String,
}

impl From<Version> for VersionLabel {
    fn from(v: Version) -> Self {
        Self {
            version: v.version,
            build_id: v.build_id,
            model: v.hw_model,
        }
    }
}
