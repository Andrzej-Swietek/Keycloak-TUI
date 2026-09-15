use std::sync::Arc;

use kct_core::ports::{Gateway, Rest};
use kct_core::{ApiPath, Method, Request, ports::Expect};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use crate::msg::{Fetch, Msg, Page};

use super::effects::{log, log_status};

pub(super) async fn load(
    token: u64,
    fetch: Fetch,
    path: ApiPath,
    query: Vec<(String, String)>,
    shape: kct_core::catalog::Shape,
    gateway: &Arc<dyn Gateway>,
    tx: &UnboundedSender<Msg>,
) {
    let result = gateway.get_json(path.clone(), query).await;
    log(tx, Method::Get, &path, &result);

    let _ = tx.send(match fetch {
        Fetch::Rows => Msg::Rows {
            token,
            result: result.map(|body| Page {
                rows: Arc::from(shape.rows(body.clone())),
                envelope: Arc::new(body),
            }),
        },
        Fetch::Options => Msg::Options {
            token,
            result: result.map(|body| Arc::from(shape.rows(body))),
        },
        Fetch::Exclusions => Msg::Exclusions {
            token,
            result: result.map(|body| Arc::from(shape.rows(body))),
        },
        Fetch::Item => Msg::Item { token, result },
    });
}

pub(super) async fn call(
    method: Method,
    path: ApiPath,
    query: Vec<(String, String)>,
    body: Option<Value>,
    title: String,
    gateway: &Arc<dyn Gateway>,
    tx: &UnboundedSender<Msg>,
) {
    let result = gateway.send(build(method, path.clone(), query, body)).await;
    log_status(tx, method, &path, result.as_ref().map(|r| r.status));

    let _ = tx.send(Msg::Done {
        title,
        result: result.map(|response| response.created_id()),
    });
}

pub(super) async fn fetch_body(
    method: Method,
    path: ApiPath,
    query: Vec<(String, String)>,
    body: Option<Value>,
    title: String,
    gateway: &Arc<dyn Gateway>,
    tx: &UnboundedSender<Msg>,
) {
    let result = gateway
        .send(build(method, path.clone(), query, body))
        .await
        .map(|response| response.body);
    log(tx, method, &path, &result);

    let _ = tx.send(Msg::Text {
        title,
        result: result.map(|body| kct_core::json::pretty(&body)),
    });
}

fn build(
    method: Method,
    path: ApiPath,
    query: Vec<(String, String)>,
    body: Option<Value>,
) -> Request {
    let request = Request::new(method, path)
        .with_query(query)
        .expecting(Expect::Json);
    match body {
        Some(body) => request.with_body(body),
        None => request,
    }
}
