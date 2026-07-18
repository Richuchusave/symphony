use symphony::{config::Config, Application};

#[tokio::main]
async fn main() -> symphony::errors::Result<()> {
    if let Some(argument) = std::env::args().nth(1) {
        match argument.as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-V" | "--version" => {
                println!("symphony {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {argument}\n\nRun 'symphony --help' for usage.");
                std::process::exit(2);
            }
        }
    }

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

fn print_help() {
    println!(
        "Symphony — terminal music player\n\nUsage: symphony [OPTIONS]\n\nOptions:\n  -h, --help       Show this help\n  -V, --version    Show the version"
    );
}
