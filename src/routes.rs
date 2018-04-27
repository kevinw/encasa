use gotham::http::response::create_response;
use gotham::router::Router;

use hyper::{Body, Response, StatusCode};

use gotham::state::{FromState, State};
use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};

use gotham::handler::{HandlerFuture, IntoHandlerError};

use futures::{future, Future, Stream};

use ::{view, update_data, mark_todo_completed, archive_finished_tasks};
use serde_json;
use mime;

header! { (XFrameOptions, "X-Frame-Options") => [String] }

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct QueryStringExtractor {
    #[serde(default)] pub context: String,
    #[serde(default)] pub project: String,
    #[serde(default)] pub search: String,
}

#[derive(Deserialize, Serialize)]
pub struct ArchiveFinishedResponse {
    pub num_archived: i32,
}


#[derive(Deserialize)]
struct TodosPost {
    hash: String,
    completed: bool,
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
        route
            .post("/actions/archive_finished")
            .to(archive_finished);
    })
}

fn archive_finished(state: State) -> Box<HandlerFuture> {
    match archive_finished_tasks() {
        Ok(num_archived) => {
            let serialized = serde_json::to_string_pretty(
                &ArchiveFinishedResponse { num_archived }).unwrap();
            let resp = create_response(&state, StatusCode::Ok,
                Some((serialized.into_bytes(), mime::APPLICATION_JSON)));
            Box::new(future::ok((state, resp)))
        },
        Err(e) => {
            Box::new(future::err((state, e.into_handler_error())))
        }
    }
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
