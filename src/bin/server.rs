use yjs_collaboration_server::application::ApplicationBootstrap;

#[volo::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 创建并启动应用
    let app = ApplicationBootstrap::new();
    app.run().await
}
