use std::fmt::{self, Write};

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use indenter::indented;
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
        log::debug!("executing {command} with {args:?}");

        let output = Command::new(command).args(args).output().await.context(format!("error executing {command} with {args:?}"))?;
        if !output.status.success() {
            let stdout = Output(&output.stdout);
            let stderr = Output(&output.stderr);
            let result = match output.status.code() {
                Some(code) => Err(anyhow!("Process exited with {code}")),
                None => Err(anyhow!("Process terminated by signal")),
            };
            return result.context(format!("error executing {command} with {args:?}\nStdout:{stdout:?}\nStderr:{stderr:?}"));
        }

        let result = String::from_utf8(output.stdout)?;
        Ok(result)
    }
}

struct Output<'a>(&'a [u8]);

impl fmt::Debug for Output<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = String::from_utf8_lossy(self.0);
        let output = output.trim_end();
        if !output.is_empty() {
            writeln!(f)?;
            write!(indented(f), "{}", output)?;
        }

        Ok(())
    }
}

pub struct CommandExecutor;

impl Executor for CommandExecutor {}
