#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use dashboard::app::ssr::AppState;
    use dashboard::app::*;
    use dashboard::fileserv::file_and_error_handler;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sqlx::sqlite::SqlitePoolOptions;

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "dashboard.db".to_owned());

    let db = SqlitePoolOptions::new()
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .create_if_missing(true)
                .foreign_keys(true)
                .filename(&db_url),
        )
        .await
        .expect("Failed to open database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Error running DB migrations");

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
