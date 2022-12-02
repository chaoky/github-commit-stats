#![feature(result_option_inspect)]

mod app;
mod histogram;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    app::App::new().run().await;
}
