use anyhow::Result;
use axum::{
    Router,
    http::header,
    response::{Html, Json},
    routing::get,
};
use std::sync::Arc;

use crate::graph::DependencyGraph;

pub async fn start(graph: DependencyGraph, port: u16, open_browser: bool) -> Result<()> {
    let graph = Arc::new(graph);

    let app = Router::new()
        .route("/", get(index))
        .route("/style.css", get(css))
        .route("/app.js", get(js))
        .route(
            "/api/graph",
            get({
                let graph = Arc::clone(&graph);
                move || {
                    let graph = Arc::clone(&graph);
                    async move { Json((*graph).clone()) }
                }
            }),
        );

    let addr = format!("127.0.0.1:{port}");

    if open_browser {
        let url = format!("http://{addr}");
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let _ = open::that(url);
        });
    }

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}

async fn css() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!("../web/style.css"),
    )
}

async fn js() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        include_str!("../web/app.js"),
    )
}
