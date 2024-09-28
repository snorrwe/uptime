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
select
    se.id,
    public_url as "public_url!",
    se."name" as "name!",
    status_code as "last_status!",
    sh."created" as "last_update!"
from status_entry as se
inner join
    (
        select status_id, status_code, created
        from status_history
        order by created desc
        limit 5
    ) as sh
    on sh.status_id = se.id
"#
    )
    .fetch_all(db)
    .await
    .map_err(|err| {
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
                                    l.as_slice()
                                        .chunk_by(|a, b| a.id == b.id)
                                        .map(move |s| {
                                            debug_assert!(!s.is_empty());
                                            let first = s.first().unwrap();
                                            view! {
                                                <li>
                                                    <a target="_blank" href=&first.public_url>
                                                        {&first.name}
                                                    </a>
                                                    " = "
                                                    <ul class="flex flex-row">
                                                        {s
                                                            .iter()
                                                            .map(|s| view! { <li>{s.last_status}</li> })
                                                            .collect_view()}
                                                    </ul>
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
