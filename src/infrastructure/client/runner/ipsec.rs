use std::io::ErrorKind;

use async_trait::async_trait;
use futures::TryStreamExt;
use indexmap::IndexMap;

use crate::{
    domain::ipsec::SA,
    infrastructure::config::env::ViciPath,
    service::{ipsec::IPsecResult, Runner},
};

type SAs = IndexMap<String, SA>;

#[derive(Clone)]
pub struct IPsecRunner {
    path: ViciPath,
}

impl IPsecRunner {
    pub fn new(path: ViciPath) -> Self {
        Self {
            path,
        }
    }

    async fn sas(&self) -> anyhow::Result<IPsecResult> {
        let mut client = match rsvici::unix::connect(self.path.as_str()).await {
            Ok(client) => client,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                log::debug!("failed to connect to strongSwan: {e}");
                return Ok(IndexMap::new());
            },
            Err(e) => {
                return Err(e.into());
            },
        };

        let stream = client.stream_request("list-sas", "list-sa", ());
        let items: Vec<SAs> = stream.try_collect().await?;
        let sas = items.into_iter().flatten().collect();

        Ok(sas)
    }
}

#[async_trait]
impl Runner for IPsecRunner {
    type Item = IPsecResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.sas().await
    }
}
