use std::{net::SocketAddr, time::Duration};

use volo_http::{
    context::ServerContext,
    http::StatusCode,
    server::{layer::TimeoutLayer, Server},
    Address,
};
use volo_http_example::create_router;
use tracing::info;
use tracing_subscriber;

fn timeout_handler(_: &ServerContext) -> (StatusCode, &'static str) {
    (StatusCode::INTERNAL_SERVER_ERROR, "Timeout!\n")
}

#[volo::main]
async fn main() {
    // Initialize tracing subscriber with environment filter and formatted output
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                // fallback level
                .add_directive("info".parse().unwrap()),
        )
        .with_target(false) // omit target field
        .with_thread_names(true)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc3339())
        .init();

    // Create a router using our refactored architecture
    let app = create_router().layer(TimeoutLayer::new(Duration::from_secs(1), timeout_handler));

    let addr = "[::]:8080".parse::<SocketAddr>().unwrap();
    let addr = Address::from(addr);

    info!("Yjs collaborative editing server running on {}", addr);

    Server::new(app).run(addr).await.unwrap();
}
