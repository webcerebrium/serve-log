mod cli;
mod logging;

use axum::body::*;
use axum::extract::{Request, State};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::*;
use axum::Router;
use clap::Parser;
use std::path::{Path, PathBuf};
use tower_http::classify::*;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeFile;
use tower_http::trace::*;
use tracing::*;

#[derive(Clone)]
struct AppState {
    web_root: PathBuf,
}

async fn middleware_handler(
    State(state): State<AppState>,
    req: Request,
    _: Next,
) -> impl IntoResponse {
    let uri = req.uri();
    let path = uri.path();
    println!(
        "\n{} {:#?} {}",
        chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
        req.method(),
        path,
    );
    let headers = req.headers().clone();
    for (k, v) in headers.iter() {
        println!("  H {:?}: {:?}", k, v);
    }
    if let Some(q) = uri.query() {
        println!("  Q {:#?}", q);
    }

    if req.method() != Method::GET
        && req.method() != Method::HEAD
        && req.method() != Method::OPTIONS
    {
        let payload = get_payload(req.into_body()).await;
        if !payload.is_empty() {
            println!("PAYLOAD:\n---\n{}\n---", String::from_utf8_lossy(&payload));
        }
    } else {
        let file_path = state.web_root.join(path);
        if std::path::Path::new(&file_path).exists() {
            // serve path
            let mut service = ServeFile::new(&file_path);
            match service.try_call(req).await {
                Ok(res) => {
                    if res.status() == 200 {
                        return res.into_response();
                    } else {
                        return "OOOK".into_response();
                    }
                }
                Err(e) => {
                    tracing::error!("ERROR: {}", e);
                    return "ERROR".into_response();
                }
            }
        }
    }
    "OK".into_response()
}

pub async fn nop() -> impl IntoResponse {
    println!("NOP");
    "unreachable".into_response()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::start_with_timings("INFO");
    let opts = cli::Cli::parse();
    let state = AppState {
        web_root: Path::new(&opts.web_root).to_path_buf(),
    };

    let app = Router::new()
        .route("/*path", axum::routing::get(nop))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware_handler,
        ))
        .with_state(state)
        .layer(axum_body_limit(1024 * 1024))
        .layer(axum_trace_full())
        .layer(axum_cors_any());
    axum_serve(&opts.bind_addr, app).await
}

// axum layer for tracing logs
pub fn axum_trace_full() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(
            DefaultMakeSpan::new()
                .level(Level::DEBUG)
                .include_headers(true),
        )
        .on_request(DefaultOnRequest::new().level(Level::TRACE))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::DEBUG)
                .include_headers(true),
        )
}

// axum layer for CORS
pub fn axum_cors_any() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

// axum layer for payload body limit
pub fn axum_body_limit(limit: usize) -> RequestBodyLimitLayer {
    RequestBodyLimitLayer::new(limit)
}

// start
pub async fn axum_serve(listen: &str, app: axum::Router) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    println!("Listening on {}", listen,);
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

#[instrument(level = "debug")]
pub fn text200(
    message: impl Into<String> + std::fmt::Display + std::fmt::Debug,
) -> impl IntoResponse {
    (StatusCode::OK, message.to_string()).into_response()
}

async fn get_payload<B>(body: B) -> Bytes
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    use http_body_util::BodyExt;
    match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => Bytes::from(vec![]),
    }
}
