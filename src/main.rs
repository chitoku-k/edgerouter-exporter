use std::process::ExitCode;

use edgerouter_exporter::di::container::Application;

#[tokio::main]
async fn main() -> ExitCode {
    match Application::start().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log::error!("failed to start application\nError: {e:?}");
            ExitCode::FAILURE
        },
    }
}
