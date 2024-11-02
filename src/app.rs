use chrono::Datelike;
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
    pub polling_url: Option<url::Url>,
}

#[derive(Deserialize)]
pub struct Config {
    pub poll_interval: Option<Duration>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct StatusDetails {
    pub id: i64,
    pub public_url: String,
    pub name: String,
    pub history: Vec<HistoryRow>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct HistoryRow {
    pub status: i64,
    pub poll_time: chrono::NaiveDateTime,
}

#[server(GetSatus, "/status/:id")]
async fn get_status_details(id: i64) -> Result<StatusDetails, ServerFnError> {
    let state = expect_context::<ssr::AppState>();
    let db = &state.db;

    let header = sqlx::query!(
        r#"
select name as "name!", public_url as "public_url!" from status_entry where id = ?
"#,
        id
    )
    .fetch_one(db)
    .await
    .map_err(|err| {
        leptos::logging::error!("Failed to load status entry: {err:?}");
        ServerFnError::<server_fn::error::NoCustomError>::ServerError(
            "Failed to load status entry".to_owned(),
        )
    })?;

    sqlx::query_as!(
        HistoryRow,
        r#"
select status_code as "status!", created as "poll_time!"
from status_history
where status_id = ?
order by created desc
"#,
        id
    )
    .fetch_all(db)
    .await
    .map_err(|err| {
        leptos::logging::error!("Failed to load status history: {err:?}");
        ServerFnError::ServerError("Failed to load status entry".to_owned())
    })
    .map(move |history| StatusDetails {
        id,
        public_url: header.public_url,
        name: header.name,
        history,
    })
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/uptime.css" />

        // sets the document title
        <Title text="Uptime" />

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors /> }.into_view()
        }>
            <Breadcrumbs />
            <main class="container mx-auto">
                <Routes>
                    <Route path="" view=HomePage />
                    <Route path="/site/:id" view=SiteDetails />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn BreadcrumbItem(name: String, url: String) -> impl IntoView {
    view! {
        <div class="flex items-center">
            <svg
                class="rtl:rotate-180 block w-3 h-3 mx-1 text-gray-400 "
                aria-hidden="true"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 6 10"
            >
                <path
                    stroke="currentColor"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="m1 9 4-4-4-4"
                />
            </svg>
            <A
                href=url
                class="ms-1 text-sm font-medium text-gray-700 hover:text-blue-600 md:ms-2 dark:text-gray-400 dark:hover:text-white"
            >
                {name}
            </A>
        </div>
    }
}

#[component]
fn Breadcrumbs() -> impl IntoView {
    let path = use_location().pathname;
    view! {
        <nav
            class="flex px-5 py-3 text-gray-700 border border-gray-200 rounded-lg bg-gray-50 dark:bg-gray-800 dark:border-gray-700"
            aria-label="Breadcrumb"
        >
            <ol class="inline-flex items-center space-x-1 md:space-x-2 rtl:space-x-reverse">
                <li class="inline-flex items-center">
                    <A
                        href="/"
                        class="inline-flex items-center text-sm font-medium text-gray-700 hover:text-blue-600 dark:text-gray-400 dark:hover:text-white"
                    >
                        <svg
                            class="w-3 h-3 me-2.5"
                            aria-hidden="true"
                            xmlns="http://www.w3.org/2000/svg"
                            fill="currentColor"
                            viewBox="0 0 20 20"
                        >
                            <path d="m19.707 9.293-2-2-7-7a1 1 0 0 0-1.414 0l-7 7-2 2a1 1 0 0 0 1.414 1.414L2 10.414V18a2 2 0 0 0 2 2h3a1 1 0 0 0 1-1v-4a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v4a1 1 0 0 0 1 1h3a2 2 0 0 0 2-2v-7.586l.293.293a1 1 0 0 0 1.414-1.414Z" />
                        </svg>
                        Home
                    </A>
                </li>
                {move || {
                    let path = path.get();
                    let mut url = String::with_capacity(path.len());
                    url.push('/');
                    path.split('/')
                        .skip(1)
                        .map(|frag| {
                            url.push_str(frag);
                            view! {
                                <li class="inline-flex items-center">
                                    <BreadcrumbItem name=frag.to_string() url=url.clone() />
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ol>
        </nav>
    }
}

#[derive(Params, Debug, PartialEq, Eq, Clone, Copy)]
struct SiteDetailsParams {
    pub id: i64,
}

#[component]
fn LoadingSpinner() -> impl IntoView {
    view! {
        <div
            class="animate-spin inline-block size-6 border-[3px] border-current border-t-transparent text-blue-600 rounded-full dark:text-blue-500"
            role="status"
            aria-label="loading"
        >
            <span class="sr-only">Loading...</span>
        </div>
    }
}

#[component]
fn SiteDetails() -> impl IntoView {
    let param = use_params::<SiteDetailsParams>();
    let id = move || param.with(|p| p.as_ref().map(|p| p.id).unwrap_or_default());

    let details = create_resource(|| (), move |_| get_status_details(id()));

    view! {
        <Suspense fallback=LoadingSpinner>
            {move || {
                match details() {
                    None => {
                        view! {
                            <h1 class="text-4xl">"Uptime"</h1>
                            <LoadingSpinner />
                        }
                            .into_view()
                    }
                    Some(Err(err)) => {
                        view! { <h1 class="text-4xl">"Error "{err.to_string()}</h1> }.into_view()
                    }
                    Some(Ok(d)) => {
                        let last = d.history.first().cloned();
                        view! {
                            <h1 class="text-4xl">"Uptime "{d.name}</h1>
                            <div class="font-medium text-blue-600 dark:text-blue-500 hover:underline">
                                <a href=d.public_url.clone() target="_blank">
                                    {d.public_url.clone()}
                                </a>
                            </div>
                            <div>
                                {move || {
                                    last.as_ref()
                                        .map(|last| {
                                            view! {
                                                <div>"Last fetch: " {last.poll_time.to_string()}</div>
                                                <div>"Status: " {last.status.to_string()}</div>
                                            }
                                        })
                                }}
                            </div>
                            <div>
                                {d
                                    .history
                                    .chunk_by(|a, b| a.poll_time.date() == b.poll_time.date())
                                    .map(|chunk| {
                                        let day = chunk[0].poll_time.date();
                                        view! {
                                            <div class="px-5 py-3 even:text-white rounded-lg even:bg-gray-600 odd:bg-gray-300 gap-2">
                                                <span>{day.to_string()}</span>
                                                <ul class="flex flex-row-reverse gap-1 flex-wrap">
                                                    {chunk.iter().map(status_pip).collect_view()}
                                                </ul>
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        }
                            .into_view()
                    }
                }
            }}
        </Suspense>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let statuses = create_resource(|| (), |_| list_statuses());

    view! {
        <h1 class="text-4xl">Uptime</h1>
        <Suspense fallback=LoadingSpinner>
            {move || {
                statuses
                    .get()
                    .map(|l| {
                        let l = l.unwrap();
                        view! {
                            <table class="table-auto">
                                <thead>
                                    <tr>
                                        <th>Name</th>
                                        <th>Uptime</th>
                                        <th>Last ping</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || {
                                        l.as_slice()
                                            .chunk_by(|a, b| a.id == b.id)
                                            .map(status_row)
                                            .collect_view()
                                    }}
                                </tbody>
                            </table>
                        }
                    })
            }}
        </Suspense>
    }
}

fn status_row(s: &[StatusRow]) -> impl IntoView {
    debug_assert!(!s.is_empty());
    let first = s.first().cloned().unwrap();
    let link = format!("/site/{}", first.id);
    let public_url = &first.public_url;
    let is_success = 200 <= first.last_status && first.last_status <= 299;
    let is_redirect = 300 <= first.last_status && first.last_status <= 399;

    let color = match (is_success, is_redirect) {
        (false, true) => "bg-yellow-200",
        (true, false) => "bg-green-200",
        (false, false) => "bg-red-200",
        (true, true) => {
            unreachable!()
        }
    };

    view! {
        <tr class=format!("{color} align-middle text-center")>
            <td class="flex flex-row gap-2">
                <A href=link>
                    <div class="cursor-pointer text-blue-600 underline decoration-gray-800 hover:opacity-80 focus:outline-none focus:opacity-80 dark:decoration-white">
                        {&first.name}
                    </div>
                </A>
                <A target="_blank" href=public_url.clone()>
                    <div class="cursor-pointer text-blue-600 underline decoration-gray-800 hover:opacity-80 focus:outline-none focus:opacity-80 dark:decoration-white">
                        "open"
                    </div>
                </A>
            </td>
            <td>{status_pip_list(s)}</td>
            <td>{first.poll_time.to_string()}</td>
        </tr>
    }
}

fn status_pip_list(s: &[StatusRow]) -> impl IntoView {
    view! {
        <ul class="flex flex-row-reverse gap-1">
            {s
                .iter()
                .map(|s| status_pip(
                    &HistoryRow {
                        status: s.last_status,
                        poll_time: s.poll_time,
                    },
                ))
                .collect_view()}
        </ul>
    }
}

fn status_pip(s: &HistoryRow) -> impl IntoView {
    const PIP: char = '\u{25AE}';

    let is_success = 200 <= s.status && s.status <= 299;
    let is_redirect = 300 <= s.status && s.status <= 399;

    let color = match (is_success, is_redirect) {
        (false, true) => "text-yellow-500",
        (true, false) => "text-green-500",
        (false, false) => "text-red-500",
        (true, true) => {
            unreachable!()
        }
    };

    view! {
        <li class=color>
            <span
                class="cursor-default text-lg hover:text-3xl"
                title=format!("{} Status: {}", s.poll_time.to_string(), s.status)
            >
                {PIP}
            </span>
        </li>
    }
}
