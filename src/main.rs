use symphony::{config::Config, Application};

#[tokio::main]
async fn main() -> symphony::errors::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "symphony=info".into()),
        )
        .init();

    let config = Config::load()?;
    config.ensure_dirs()?;

    let mut app = Application::new(config).await?;
    app.run().await?;

    Ok(())
}
