#[macro_use]
extern crate log;

use std::process::exit;

use edgerouter_exporter::di::container::Application;

#[tokio::main]
async fn main() {
    if let Err(e) = Application::start().await {
        error!("Failed to start application: {e}");
        exit(1);
    }
}
