// Binary executable entry point for the Yjs Collaboration Server
//
// This is the main entry point for the Yjs Collaboration Server executable.
// It initializes the application bootstrap and starts the server.

use yjs_collaboration_server_application::ApplicationBootstrap;

#[volo::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create and run the application bootstrap
    let bootstrap = ApplicationBootstrap::new();
    bootstrap.run().await
}
