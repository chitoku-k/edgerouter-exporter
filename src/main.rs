use edgerouter_exporter::{
    di::container::Application,
    infrastructure::config::env,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = env::get()?;
    Application::start(&config).await?;
    Ok(())
}
