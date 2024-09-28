use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
pub mod ssr {

    use axum::extract::FromRef;
    use leptos::LeptosOptions;
    use sqlx::SqlitePool;

    #[derive(Debug, FromRef, Clone)]
    pub struct AppState {
        pub db: SqlitePool,
        pub leptos_options: LeptosOptions,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct StatusRow {
    pub id: i64,
    pub public_url: String,
    pub name: String,
    pub last_status: i64,
    pub last_update: chrono::NaiveDateTime,
}

#[server(GetSatuses, "/status")]
async fn list_statuses() -> Result<Vec<StatusRow>, ServerFnError> {
    let state = expect_context::<ssr::AppState>();
    let db = &state.db;
    sqlx::query_as!(
        StatusRow,
        r#"
SELECT se.id, se."name" AS "name!", sh."created" as "last_update!", status_code as "last_status!", public_url FROM status_entry AS se
INNER JOIN (
    SELECT status_id, status_code, MAX(created) as created
    FROM status_history
    GROUP BY status_id
) AS sh
ON sh.status_id = se.id
"#
    ).fetch_all(db).await.map_err(|err|{
            leptos::logging::error!("Failed to load status entries: {err:?}");
            ServerFnError::ServerError("Failed to load status entries".to_owned())
        })
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/dashboard.css" />

        // sets the document title
        <Title text="Dashboard" />

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors /> }.into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let statuses = create_resource(|| (), |_| list_statuses());

    view! {
        <h1 class="text-xl">Dashboard</h1>
        <Suspense fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || {
                statuses
                    .get()
                    .map(|l| {
                        let l = l.unwrap();
                        view! {
                            <ul>
                                {move || {
                                    l.iter()
                                        .map(move |s| {
                                            view! {
                                                <li>
                                                    <a target="_blank" href=&s.public_url>
                                                        {&s.name} " = " {s.last_status}
                                                    </a>
                                                </li>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </ul>
                        }
                    })
            }}
        </Suspense>
    }
}
