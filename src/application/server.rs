use std::{future::Future, net::Ipv6Addr, sync::Arc};

use anyhow::Context;
use axum::{routing::get, Router};
use tokio::net::TcpListener;
#[cfg(feature = "tls")]
use {
    hyper::service::service_fn,
    hyper_util::{rt::tokio::{TokioExecutor, TokioIo}, server::conn::auto::Builder},
    notify::Watcher,
    openssl::ssl::{self, AlpnError, SslContext, SslFiletype, SslMethod},
    tls_listener::TlsListener,
    tokio::{
        sync::mpsc::unbounded_channel,
        time::{sleep, Duration},
        select,
    },
    tower_service::Service,
};

use crate::application::metrics;

pub trait Controller<T> {
    fn handle(&self) -> impl Future<Output = anyhow::Result<T>> + Send;
}

pub struct Engine<MetricsController> {
    port: u16,
    tls: Option<(String, String)>,
    metrics_controller: MetricsController,
}

impl<MetricsController> Engine<MetricsController>
where
    MetricsController: Controller<String> + Send + Sync + 'static,
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
            .with_state(Arc::new(self.metrics_controller));

        let addr = (Ipv6Addr::UNSPECIFIED, self.port);
        let app = Router::new()
            .nest("/healthz", health)
            .nest("/metrics", metrics);

        let listener = TcpListener::bind(addr).await?;
        match self.tls {
            #[cfg(not(feature = "tls"))]
            Some(_) => {
                panic!("TLS is not enabled.");
            },
            #[cfg(feature = "tls")]
            Some((tls_cert, tls_key)) => {
                bind_tls(app, listener, tls_cert, tls_key).await?;
            },
            None => {
                bind(app, listener).await?;
            },
        }

        Ok(())
    }
}

#[cfg(feature = "tls")]
fn acceptor(tls_cert: &str, tls_key: &str) -> anyhow::Result<SslContext> {
    let mut builder = SslContext::builder(SslMethod::tls_server())?;
    builder
        .set_certificate_chain_file(tls_cert)
        .context("error loading TLS certificate")?;
    builder
        .set_private_key_file(tls_key, SslFiletype::PEM)
        .context("error loading TLS private key")?;
    builder
        .set_alpn_select_callback(|_, client| {
            ssl::select_next_proto(b"\x02h2\x08http/1.1", client).ok_or(AlpnError::NOACK)
        });

    Ok(builder.build())
}

#[cfg(feature = "tls")]
async fn bind_tls(app: Router, listener: TcpListener, tls_cert: String, tls_key: String) -> anyhow::Result<()> {
    let (tx, mut rx) = unbounded_channel();

    let mut watcher = notify::recommended_watcher(move |event| {
        if let Ok(event) = event {
            tx.send(event).expect("error reloading TLS certificate");
        }
    })?;
    watcher.watch(tls_cert.as_ref(), notify::RecursiveMode::NonRecursive)?;

    let mut listener = TlsListener::new(acceptor(&tls_cert, &tls_key)?, listener);
    let service = service_fn(move |request| {
        app.clone().call(request)
    });

    loop {
        select! {
            stream = listener.accept() => {
                match stream.context("error accepting TLS listener")? {
                    Ok((stream, _remote)) => {
                        let http = Builder::new(TokioExecutor::new());
                        let io = TokioIo::new(stream);
                        let service = service.clone();
                        tokio::spawn(async move {
                            if let Err(e) = http.serve_connection(io, service).await {
                                log::debug!("{}", e);
                            }
                        });
                    },
                    Err(e) => {
                        log::debug!("{}", e);
                    },
                }
            },
            event = rx.recv() => {
                if event.filter(|e| e.kind.is_modify()).is_none() {
                    continue;
                }
                sleep(Duration::from_millis(200)).await;

                match acceptor(&tls_cert, &tls_key) {
                    Ok(acceptor) => {
                        listener.replace_acceptor(acceptor);
                    },
                    Err(e) => {
                        log::warn!("{:?}", e);
                    },
                }
            },
        }
    }
}

async fn bind(app: Router, listener: TcpListener) -> anyhow::Result<()> {
    axum::serve(listener, app)
        .await
        .context("error starting server")
}
