use actix;
use actix_web::{pred, http, server, App, Query, HttpResponse,
    Json, Path, middleware, Error, HttpRequest};
use actix_web::http::Method;
use failure;
use homepage_view::{render, SearchParams};
use env_logger;
use std;

use homepage_data::{update_data, mark_todo_completed, archive_finished_tasks,
    update_deadlines};

fn _render_index(files_to_include: &Vec<String>, search_params: &SearchParams) -> Result<HttpResponse, failure::Error> {
    let cached_data = update_data(files_to_include)?;
    let html = render(&cached_data, search_params)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

fn update_deadlines_route(_info: Path<()>) -> Result<HttpResponse, failure::Error> {
    update_deadlines()?;
    _render_index(&vec![], &SearchParams::default())
}

fn index(query: Query<IndexQuery>) -> Result<HttpResponse, failure::Error> {
    let mut files_to_include = vec![];
    if !query.file.is_empty() {
        files_to_include.push(query.file.clone());
    }
    _render_index(&files_to_include, &query.to_search_params())
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
    #[serde(default)] pub file: String,
}

impl IndexQuery {
    fn to_search_params(&self) -> SearchParams {
        SearchParams {
            context: self.context.clone(),
            project: self.project.clone(),
            search: self.search.clone(),
            sort_by: self.sort_by.clone(),
        }
    }
}

fn p404(req: HttpRequest) -> Result<HttpResponse, Error> {
    println!("{:?}lll", req);
    Ok(HttpResponse::NotFound().content_type("text/plain").body("Not Found"))
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
                    //.limit(4096); // <- limit size of the payload
                    ;
            })
            .route("/actions/archive_finished", http::Method::POST, archive_finished)
            .route("/update_deadlines", http::Method::GET, update_deadlines_route)
            .route("/", http::Method::GET, index)
            .default_resource(|r| {
                // 404 for GET request
                r.method(Method::GET);//.f(p404);

                // all requests that are not `GET`
                r.route().filter(pred::Not(pred::Get())).f(|_req| HttpResponse::MethodNotAllowed());
            })
    }).bind(&addr)
        .unwrap()
        .shutdown_timeout(1)
        .start();

    println!("Started http server: {}", addr);
    let _ = sys.run();
}
