use std::{net::Ipv6Addr, sync::Arc};

use async_trait::async_trait;
use warp::{Filter, Reply};

#[async_trait]
pub trait Controller {
    async fn handle(&self) -> Box<dyn Reply>;
}

#[derive(Clone)]
pub struct Engine<MetricsController>
where
    MetricsController: Controller + Send + Sync + Clone + 'static,
{
    port: u16,
    tls: Option<(String, String)>,
    metrics_controller: MetricsController,
}

impl<MetricsController> Engine<MetricsController>
where
    MetricsController: Controller + Send + Sync + Clone + 'static,
{
    pub fn new(
        port: u16,
        tls_cert: Option<String>,
        tls_key: Option<String>,
        metrics_controller: MetricsController,
    ) -> Self {
        let tls = Option::zip(tls_cert, tls_key);
        Self {
            port,
            tls,
            metrics_controller,
        }
    }

    pub async fn start(self: Arc<Self>) {
        let port = self.port;
        let tls = self.tls.clone();

        let metrics = warp::path("metrics").then(move || {
            let engine = self.clone();
            async move {
                engine.metrics_controller.handle().await
            }
        });

        let server = warp::serve(metrics);
        match tls {
            #[cfg(not(feature="tls"))]
            Some(_) => {
                panic!("TLS is not enabled.");
            },
            #[cfg(feature="tls")]
            Some((tls_cert, tls_key)) => {
                server
                    .tls()
                    .cert_path(tls_cert)
                    .key_path(tls_key)
                    .run((Ipv6Addr::UNSPECIFIED, port)).await
            },
            None => {
                server
                    .run((Ipv6Addr::UNSPECIFIED, port)).await
            },
        }
    }
}
