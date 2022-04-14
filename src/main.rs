mod poker_server;
mod web_server;

use bastion::prelude::*;

use std::error::Error;

use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};
use tracing_tree::HierarchicalLayer;

fn start_bastion() {
    let config = Config::new().hide_backtraces();
    Bastion::init_with(config);
    Bastion::start();
}

fn setup_tracing() -> Result<(), Box<dyn Error>> {
    Registry::default()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("POKER_LOG")
                .from_env()?,
        )
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(console_subscriber::spawn())
        .with(tracing_subscriber::fmt::layer().json())
        .init();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_tracing()?;
    start_bastion();

    let poker_server = poker_server::run()?;

    let _web_server = web_server::run(poker_server).await?;

    Bastion::block_until_stopped();
    Ok(())
}
