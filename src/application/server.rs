use std::{net::Ipv6Addr, sync::Arc};

use async_trait::async_trait;
use axum::{
    extract::Extension,
    routing::get,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;

use crate::application::metrics;

#[async_trait]
pub trait Controller<T> {
    async fn handle(&self) -> anyhow::Result<T>;
}

#[derive(Clone)]
pub struct Engine<MetricsController> {
    port: u16,
    tls: Option<(String, String)>,
    metrics_controller: MetricsController,
}

impl<MetricsController> Engine<MetricsController>
where
    MetricsController: Controller<String> + Send + Sync + Clone + 'static,
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

    pub async fn start(self) -> anyhow::Result<()> {
        let health = Router::new()
            .route("/", get(|| async { "OK" }));

        let metrics = Router::new()
            .route("/", get(metrics::handle::<MetricsController>))
            .layer(Extension(Arc::new(self.metrics_controller)));

        let app = Router::new()
            .nest("/healthz", health)
            .nest("/metrics", metrics);
        let addr = (Ipv6Addr::UNSPECIFIED, self.port).into();

        match &self.tls {
            #[cfg(not(feature = "tls"))]
            Some(_) => {
                panic!("TLS is not enabled.");
            },
            #[cfg(feature = "tls")]
            Some((tls_cert, tls_key)) => {
                axum_server::bind_rustls(addr, RustlsConfig::from_pem_file(tls_cert, tls_key).await?)
                    .serve(app.into_make_service())
                    .await?;
            },
            None => {
                axum_server::bind(addr)
                    .serve(app.into_make_service())
                    .await?;
            },
        }

        Ok(())
    }
}
