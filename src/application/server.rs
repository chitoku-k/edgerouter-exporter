use std::{net::Ipv6Addr, sync::Arc};
#[cfg(feature = "tls")]
use std::{
    sync::mpsc::channel,
    time::Duration,
};

use async_trait::async_trait;
use axum::{
    extract::Extension,
    routing::get,
    Router,
};
#[cfg(feature = "tls")]
use axum_server::tls_rustls::RustlsConfig;
#[cfg(feature = "tls")]
use notify::{DebouncedEvent, Watcher};
#[cfg(feature = "tls")]
use tokio::task::JoinHandle;

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
                let config = RustlsConfig::from_pem_file(&tls_cert, &tls_key).await?;
                enable_auto_reload(config.clone(), tls_cert, tls_key);

                axum_server::bind_rustls(addr, config)
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

#[cfg(feature = "tls")]
fn enable_auto_reload(config: RustlsConfig, tls_cert: String, tls_key: String) -> JoinHandle<anyhow::Result<()>> {
    let (tx, rx) = channel();

    tokio::spawn(async move {
        let mut watcher = notify::watcher(tx, Duration::from_secs(1))?;
        watcher.watch(&tls_cert, notify::RecursiveMode::NonRecursive)?;

        loop {
            let event = rx.recv()?;
            if let DebouncedEvent::Write(_) = event {
                config.reload_from_pem_file(&tls_cert, &tls_key).await?;
            }
        }
    })
}
