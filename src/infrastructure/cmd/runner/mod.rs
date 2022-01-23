use async_trait::async_trait;
use tokio::process::Command;

pub mod bgp;
pub mod ddns;
pub mod load_balance;
pub mod pppoe;
pub mod version;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Executor {
    async fn output<'a>(&self, command: &str, args: &[&'a str]) -> anyhow::Result<String> {
        let output = Command::new(command).args(args).output().await?;
        let result = String::from_utf8(output.stdout)?;
        Ok(result)
    }
}

#[derive(Clone)]
pub struct CommandExecutor;

impl Executor for CommandExecutor {}
