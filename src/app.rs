use std::time::Duration;

use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Entry {
    pub name: String,
    pub public_url: url::Url,
    pub internal_url: Option<url::Url>,
}

#[derive(Deserialize)]
pub struct Config {
    pub poll_internal: Option<Duration>,
    pub entries: Vec<Entry>,
}

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
    pub poll_time: chrono::NaiveDateTime,
}

#[server(GetSatuses, "/status")]
async fn list_statuses() -> Result<Vec<StatusRow>, ServerFnError> {
    let state = expect_context::<ssr::AppState>();
    let db = &state.db;
    sqlx::query_as!(
        StatusRow,
        r#"
with
    ranked_history as (
        select
            se.id,
            public_url as "public_url!",
            se."name" as "name!",
            status_code as "last_status!",
            sh."created" as "poll_time!",
            row_number() over (partition by se.id order by sh.created desc) as rn
        from status_entry as se
        inner join
            (select status_id, status_code, created from status_history) as sh
            on sh.status_id = se.id
    )
select id, "public_url!", "name!", "last_status!", "poll_time!"

from ranked_history
where rn <= 10
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
        <Script src="/preline/preline.js"></Script>

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
            view! {
                <div
                    class="animate-spin inline-block size-6 border-[3px] border-current border-t-transparent text-blue-600 rounded-full dark:text-blue-500"
                    role="status"
                    aria-label="loading"
                >
                    <span class="sr-only">Loading...</span>
                </div>
            }
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
                                        .map(status_li)
                                        .collect_view()
                                }}
                            </ul>
                        }
                    })
            }}
        </Suspense>
    }
}

fn status_li(s: &[StatusRow]) -> impl IntoView {
    debug_assert!(!s.is_empty());
    let first = s.first().unwrap();
    view! {
        <li class="flex flex-row">
            <a target="_blank" href=&first.public_url>
                {&first.name}
            </a>
            " = "
            <ul class="flex flex-row-reverse gap-1">
                {s
                    .iter()
                    .map(|s| {
                        view! {
                            <li class="hs-tooltip [--trigger:hover] inline-block">

                                {s.last_status}
                                <span
                                    class="hs-tooltip-content hs-tooltip-shown:opacity-100 hs-tooltip-shown:visible opacity-0 transition-opacity inline-block absolute invisible z-10 py-3 px-4 bg-white border text-sm text-gray-600 rounded-lg shadow-md dark:bg-neutral-900 dark:border-neutral-700 dark:text-neutral-400"
                                    role="tooltip"
                                >
                                    {s.poll_time.to_string()}
                                </span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </li>
    }
}
