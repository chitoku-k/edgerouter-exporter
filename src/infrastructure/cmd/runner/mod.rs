use async_trait::async_trait;
use tokio::process::Command;

pub mod bgp;
pub mod ddns;
pub mod load_balance;
pub mod pppoe;
pub mod version;

#[async_trait]
pub trait Executor {
    async fn output(&self, command: &str, args: &[&str]) -> anyhow::Result<String> {
        let output = Command::new(command).args(args).output().await?;
        let result = String::from_utf8(output.stdout)?;
        Ok(result)
    }
}
