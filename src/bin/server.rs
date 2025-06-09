use yjs_collaboration_server::application::ApplicationBootstrap;

#[volo::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create and start the application
    let app = ApplicationBootstrap::new();
    app.run().await
}
