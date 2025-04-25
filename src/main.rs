#[cfg(feature = "ssr")]
#[derive(clap_derive::Parser)]
struct Args {
    #[clap(long, short, default_value = "uptime.toml")]
    pub config: std::path::PathBuf,
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::str::FromStr as _;
    use std::time::Duration;

    use axum::Router;
    use clap::Parser;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sqlx::sqlite::SqlitePoolOptions;
    use tracing_subscriber::prelude::*;
    use uptime::app::*;
    use uptime::fileserv::file_and_error_handler;
    use uptime::status_check::poll_statuses;
    use uptime::{app::ssr::AppState, status_check::init_statuses};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .expect("Failed to init tracing");

    let args = Args::parse();

    let config = std::fs::read_to_string(&args.config).expect("Failed to read config file");

    let config: Config = toml::from_str(&config).expect("Failed to parse config file");

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "uptime.db".to_owned());

    let opts = sqlx::sqlite::SqliteConnectOptions::from_str(&db_url)
        .expect("Failed to parse DATABASE_URL")
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .create_if_missing(true)
        .foreign_keys(true);

    if let Some(p) = opts.get_filename().parent() {
        tracing::debug!(?p, "Creating parent directories");
        std::fs::create_dir_all(p).expect("Failed to create dirs")
    }

    let db = SqlitePoolOptions::new()
        .connect_with(opts)
        .await
        .expect("Failed to open database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Error running DB migrations");

    init_statuses(&db, &config.entries)
        .await
        .expect("Failed to setup database");

    let interval = config.poll_interval.unwrap_or(Duration::from_secs(30));
    tracing::info!("Polling every {interval:?}");
    tokio::spawn(poll_statuses(db.clone(), interval));

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // build our application with a route
    let state = AppState { db, leptos_options };

    let app = Router::new()
        .leptos_routes(&state, routes, App)
        .fallback(file_and_error_handler)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

#[cfg(feature = "ssr")]
async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
