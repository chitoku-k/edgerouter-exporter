use std::process::exit;

use edgerouter_exporter::di::container::Application;

#[tokio::main]
async fn main() {
    if let Err(e) = Application::start().await {
        log::error!("failed to start application\nError: {e:?}");
        exit(1);
    }
}
