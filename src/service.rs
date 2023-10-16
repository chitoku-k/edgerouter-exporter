use std::future::Future;

pub mod bgp;
pub mod ddns;
pub mod interface;
pub mod ipsec;
pub mod load_balance;
pub mod pppoe;
pub mod version;

pub trait Runner {
    type Item;

    fn run(&self) -> impl Future<Output = anyhow::Result<Self::Item>> + Send;
}
