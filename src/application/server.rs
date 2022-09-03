use std::{net::Ipv6Addr, sync::Arc};
#[cfg(feature = "tls")]
use std::sync::mpsc::channel;

use anyhow::Context;
use async_trait::async_trait;
use axum::{
    extract::Extension,
    routing::get,
    Router,
};
#[cfg(feature = "tls")]
use {
    axum_server::tls_rustls::RustlsConfig,
    notify::Watcher,
    tokio::task::JoinHandle,
};

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

        let addr = (Ipv6Addr::UNSPECIFIED, self.port).into();
        let app = Router::new()
            .nest("/healthz", health)
            .nest("/metrics", metrics);

        match self.tls {
            #[cfg(not(feature = "tls"))]
            Some(_) => {
                panic!("TLS is not enabled.");
            },
            #[cfg(feature = "tls")]
            Some((tls_cert, tls_key)) => {
                let config = RustlsConfig::from_pem_file(&tls_cert, &tls_key).await.context("error loading TLS certificates")?;
                enable_auto_reload(config.clone(), tls_cert, tls_key);

                axum_server::bind_rustls(addr, config)
                    .serve(app.into_make_service())
                    .await
                    .context("error starting server")?;
            },
            None => {
                axum_server::bind(addr)
                    .serve(app.into_make_service())
                    .await
                    .context("error starting server")?;
            },
        }

        Ok(())
    }
}

#[cfg(feature = "tls")]
fn enable_auto_reload(config: RustlsConfig, tls_cert: String, tls_key: String) -> JoinHandle<anyhow::Result<()>> {
    let (tx, rx) = channel();

    tokio::spawn(async move {
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(tls_cert.as_ref(), notify::RecursiveMode::NonRecursive)?;

        loop {
            let event = rx.recv()?.context("error watching updates on TLS certificates")?;
            if event.kind.is_modify() {
                config.reload_from_pem_file(&tls_cert, &tls_key).await?;
            }
        }
    })
}
