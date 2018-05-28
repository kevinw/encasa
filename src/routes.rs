use actix;
use actix_web::{http, server, App, Query, HttpResponse, Json, Path, middleware};

use failure;
use homepage_data::{update_data, mark_todo_completed, archive_finished_tasks};
use serde_json;
use homepage_view;
use env_logger;
use std;

fn index(query: Query<IndexQuery>) -> Result<HttpResponse, failure::Error> {
    let cached_data = update_data()?;
    let serialized = serde_json::to_string_pretty(&cached_data).unwrap();
    let html = homepage_view::render(&cached_data, &serialized, &query.to_search_params())?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

#[derive(Deserialize, Serialize, Debug)]
struct TodosPost {
    hash: String,
    completed: bool,
}

#[derive(Serialize)]
struct TodosPostResponse {
    hash: String,
}

#[derive(Deserialize, Serialize)]
pub struct ArchiveFinishedResponse {
    pub num_archived: u32,
}

fn post_todos(data: Json<TodosPost>) -> Result<HttpResponse, failure::Error> {
    let new_hash = mark_todo_completed(&data.hash, data.completed)?;
    if new_hash.is_empty() {
        Ok(HttpResponse::NotFound().body("hash not found"))
    } else {
        Ok(HttpResponse::Ok().json(TodosPostResponse { hash: new_hash }))
    }
}

fn archive_finished(_info: Path<()>) -> Result<HttpResponse, failure::Error> {
    let num_archived = archive_finished_tasks()?;
    Ok(HttpResponse::Ok().json(ArchiveFinishedResponse { num_archived }))
}

#[derive(Deserialize)]
pub struct IndexQuery {
    #[serde(default)] pub context: String,
    #[serde(default)] pub project: String,
    #[serde(default)] pub search: String,
    #[serde(default)] pub sort_by: String,
}

impl IndexQuery {
    fn to_search_params(&self) -> homepage_view::SearchParams {
        homepage_view::SearchParams {
            context: self.context.clone(),
            project: self.project.clone(),
            search: self.search.clone(),
            sort_by: self.sort_by.clone(),
        }
    }
}

pub fn run_server(port_str: &str) {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let sys = actix::System::new("encasa-dev-server");
    let addr = format!("127.0.0.1:{}", port_str);

    server::new(|| {
        App::new()
            // enable logger
            .middleware(middleware::Logger::default())
            .resource("/todos", |r| {
                r.method(http::Method::POST)
                    .with(post_todos)
                    .limit(4096); // <- limit size of the payload
            })
            .route("/actions/archive_finished", http::Method::POST, archive_finished)
            .route("/", http::Method::GET, index)
    }).bind(&addr)
        .unwrap()
        .shutdown_timeout(1)
        .start();

    println!("Started http server: {}", addr);
    let _ = sys.run();
}
