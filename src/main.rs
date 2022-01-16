use edgerouter_exporter::{
    di::container::Application,
    infrastructure::config::env,
    service::Runner,
};
use tokio::try_join;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = env::get()?;
    let application = Application::new(&config);

    let (
        (bgp4, bgp6),
        ddns,
        load_balance,
        pppoe,
        version,
    ) = try_join!(
        application.bgp_runner.run(),
        application.ddns_runner.run(),
        application.load_balance_runner.run(),
        application.pppoe_runner.run(),
        application.version_runner.run(),
    )?;

    println!("BGP (IPv4): {bgp4:#?}");
    println!("BGP (IPv6): {bgp6:#?}");
    println!("DDNS: {ddns:#?}");
    println!("Load Balance: {load_balance:#?}");
    println!("PPPoE: {pppoe:#?}");
    println!("Version: {version:#?}");

    Ok(())
}
