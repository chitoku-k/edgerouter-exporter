use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Runner {
    type Item;

    async fn run(&self) -> Result<Self::Item>;
}
