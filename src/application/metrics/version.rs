use prometheus_client::{
    encoding::text::Encode,
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};

use crate::{
    application::metrics::Collector,
    domain::version::Version,
    service::version::VersionResult,
};

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

impl Collector for VersionResult {
    fn collect(self, registry: &mut Registry) {
        let info = Family::<VersionLabel, Gauge>::default();
        registry.register(
            "edgerouter_info",
            "Version info",
            Box::new(info.clone()),
        );

        info.get_or_create(&self.into()).set(1);
    }
}
