use std::{fmt::{self, Write}, future::Future};

use anyhow::{anyhow, Context};
use indenter::indented;
use tokio::process::Command;

pub mod bgp;
pub mod ddns;
pub mod load_balance;
pub mod pppoe;
pub mod version;

#[cfg(test)]
mockall::mock! {
    pub(super) Executor {}

    impl Executor for Executor {
        fn output<'a>(&self, command: &str, args: &[&'a str]) -> impl Future<Output = anyhow::Result<String>> + Send;
    }
}

pub trait Executor {
    fn output(&self, command: &str, args: &[&str]) -> impl Future<Output = anyhow::Result<String>> + Send {
        log::debug!("executing {command} with {args:?}");

        async move {
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
}

struct Output<'a>(&'a [u8]);

impl fmt::Debug for Output<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = String::from_utf8_lossy(self.0);
        let output = output.trim_end();
        if !output.is_empty() {
            writeln!(f)?;
            write!(indented(f), "{output}")?;
        }

        Ok(())
    }
}

pub struct CommandExecutor;

impl Executor for CommandExecutor {}
