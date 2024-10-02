#[cfg(feature = "ssr")]
#[derive(clap_derive::Parser)]
struct Args {
    #[clap(long, short, default_value = "dashboard.toml")]
    pub config: std::path::PathBuf,
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::str::FromStr;
    use std::time::Duration;

    use axum::Router;
    use clap::Parser;
    use dashboard::app::*;
    use dashboard::fileserv::file_and_error_handler;
    use dashboard::status_check::poll_statuses;
    use dashboard::{app::ssr::AppState, status_check::init_statuses};
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sqlx::sqlite::SqlitePoolOptions;

    let args = Args::parse();

    let config = std::fs::read_to_string(&args.config).expect("Failed to read config file");

    let config: Config = toml::from_str(&config).expect("Failed to parse config file");

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "dashboard.db".to_owned());
    let db_path = std::path::PathBuf::from_str(&db_url).unwrap();

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create directories for database");
    }

    let db = SqlitePoolOptions::new()
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .create_if_missing(true)
                .foreign_keys(true)
                .filename(&db_path),
        )
        .await
        .expect("Failed to open database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Error running DB migrations");

    init_statuses(&db, &config.entries)
        .await
        .expect("Failed to setup database");

    let interval = config.poll_internal.unwrap_or(Duration::from_secs(30));
    logging::log!("Polling every {interval:?}");
    tokio::spawn(poll_statuses(db.clone(), interval));

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await.unwrap();
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
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
