use std::process::Command;

use anyhow::Result;

pub mod bgp;
pub mod ddns;
pub mod load_balance;
pub mod pppoe;
pub mod version;

pub trait Executor {
    fn output(&self, command: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(command).args(args).output()?;
        let result = String::from_utf8(output.stdout)?;
        Ok(result)
    }
}
