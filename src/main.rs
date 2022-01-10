use anyhow::Result;
use edgerouter_exporter::{
    di::container::Application,
    infrastructure::config::env,
    service::Runner,
};

fn main() -> Result<()> {
    let config = env::get()?;
    let application = Application::new(&config);

    let (bgp4, bgp6) = application.bgp_runner.run()?;
    let ddns = application.ddns_runner.run()?;
    let load_balance = application.load_balance_runner.run()?;
    let pppoe = application.pppoe_runner.run()?;
    let version = application.version_runner.run()?;

    println!("BGP (IPv4): {bgp4:#?}");
    println!("BGP (IPv6): {bgp6:#?}");
    println!("DDNS: {ddns:#?}");
    println!("Load Balance: {load_balance:#?}");
    println!("PPPoE: {pppoe:#?}");
    println!("Version: {version:#?}");

    Ok(())
}
