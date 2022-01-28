use async_trait::async_trait;

pub mod bgp;
pub mod ddns;
pub mod interface;
pub mod load_balance;
pub mod pppoe;
pub mod version;

#[async_trait]
pub trait Runner {
    type Item;

    async fn run(&self) -> anyhow::Result<Self::Item>;
}
