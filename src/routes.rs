use gotham::http::response::create_response;
use gotham::router::Router;

use hyper::{Body, Response, StatusCode};

use gotham::state::{FromState, State};
use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};

use gotham::handler::{HandlerFuture, IntoHandlerError};

use futures::{future, Future, Stream};

use ::{view};
use serde_json;
use mime;

header! { (XFrameOptions, "X-Frame-Options") => [String] }

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct QueryStringExtractor {
    #[serde(default)] pub context: String,
    #[serde(default)] pub project: String,
    #[serde(default)] pub search: String,
}

use ::{update_data, mark_todo_completed};

#[derive(Deserialize)]
struct TodosPost {
    hash: String,
    completed: bool,
}

use std::io::{BufReader, Read};
use std::fs::File;

fn get_file_bytes(path: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    BufReader::new(File::open(path).unwrap()).read_to_end(&mut buf).unwrap();
    buf
}

fn get_font(path: &str) -> Option<(Vec<u8>, mime::Mime)> {
    let mime_type: mime::Mime = "font/woff2".parse().unwrap();
    Some((get_file_bytes(path), mime_type))
}

#[allow(non_snake_case)]
pub fn TK3iWkUHHAIjg752GT8G(state: State) -> (State, Response) {
    let resp = create_response(&state, StatusCode::Ok, get_font("static/TK3iWkUHHAIjg752GT8G.woff2"));
    (state, resp)
}

#[allow(non_snake_case)]
pub fn TK3iWkUHHAIjg752HT8Ghe4(state: State) -> (State, Response) {
    let resp = create_response(&state, StatusCode::Ok, get_font("static/TK3iWkUHHAIjg752GT8Ghe4.woff2"));
    (state, resp)
}

pub fn router() -> Router {
    build_simple_router(|route| {
        route
            .post("/todos")
            .to(post_todos);
        route
            .get("/")
            .with_query_string_extractor::<QueryStringExtractor>()
            .to(index);
        route.get("/fonts/TK3iWkUHHAIjg752GT8G.woff2")
            .to(TK3iWkUHHAIjg752GT8G);
        route.get("/fonts/TK3iWkUHHAIjg752GT8Ghe4.woff2")
            .to(TK3iWkUHHAIjg752HT8Ghe4);
    })
}


fn post_todos(mut state: State) -> Box<HandlerFuture> {
    Box::new(Body::take_from(&mut state)
        .concat2()
        .then(|full_body| match full_body {
            Ok(valid_body) => {
                // TODO: instead of .unwrap(), everything should be in a context
                // where server errors become 500 responses.
                let body_content = String::from_utf8(valid_body.to_vec()).unwrap();
                let data: TodosPost = serde_json::from_str(&body_content).unwrap();
                match mark_todo_completed(&data.hash, data.completed) {
                    Ok(new_hash) => {
                        let res = if !new_hash.is_empty() {
                            let mut response_string = String::from("{\"hash\": \"");
                            response_string.push_str(&new_hash);
                            response_string.push_str("\"}");
                            create_response(&state, StatusCode::Ok,
                                Some((response_string.into_bytes(), mime::APPLICATION_JSON)))
                        } else {
                            create_response(&state, StatusCode::NotFound, None)
                        };
                        future::ok((state, res))
                    },
                    Err(msg) => {
                        future::err((state, msg.into_handler_error()))
                    }
                }
            }
            Err(e) => future::err((state, e.into_handler_error())),
        }))
}

pub fn index(mut state: State) -> (State, Response) {
    let cached_data = update_data().unwrap();
    let serialized = serde_json::to_string_pretty(&cached_data).unwrap();
    let html_bytes = view::render(&cached_data, &serialized, &QueryStringExtractor::take_from(&mut state)).into_bytes();
    let mut res = create_response(&state, StatusCode::Ok, Some((html_bytes, mime::TEXT_HTML)));
    res.headers_mut().set(XFrameOptions("ALLOW FROM file://".to_owned()));
    (state, res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;

    #[test]
    fn receive_hello_world_response() {
        let test_server = TestServer::new(|| Ok(index)).unwrap();
        let response = test_server
            .client()
            .get("http://localhost")
            .perform()
            .unwrap();

        let status_code = response.status();

        let body = response.read_body().unwrap();
        assert!(body.len() > 0);

        let body_str = String::from_utf8(body).unwrap();
        assert_eq!(status_code, StatusCode::Ok, "body was {}", body_str);

    }
}
