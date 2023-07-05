use std::{
    ffi::OsStr,
    fmt::Display,
    fs,
    net::SocketAddr,
    path::{self, PathBuf},
    sync::Arc,
};

use axum::{
    extract::{Path, State},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use tokio::sync::RwLock;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

struct Server {
    template: String,
}

#[derive(Debug)]
struct LoadError(String);

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

struct Page {
    content_type: String,
    contents: String,
}

fn detect_content_type(path: &path::Path) -> &str {
    match path
        .extension()
        .unwrap_or(OsStr::new("html"))
        .to_str()
        .unwrap_or("html")
    {
        "css" => return "text/css",
        "html" => return "text/html",
        "js" => return "application/js",
        _ => return "text/plain",
    }
}

impl Server {
    fn read_file(&self, path: &path::Path) -> Result<String, LoadError> {
        tracing::info!("trying to load {:?}", path);
        let data = fs::read(path::Path::new("example_site").join(path))
            .map_err(|f| LoadError(f.to_string()))?;
        String::from_utf8(data).map_err(|e| LoadError(e.to_string()))
    }

    fn read_request_file(&self, path: &path::Path) -> Result<Page, LoadError> {
        let path: PathBuf = if path.extension().is_some() {
            path.into()
        } else {
            path.with_extension("html")
        };

        let paths_to_check = vec![
            path::Path::new("pages").join(&path),
            path::Path::new("public").join(&path),
        ];

        for path in paths_to_check {
            let file = self.read_file(&path);
            if file.is_ok() {
                let contents = file.unwrap();
                return Ok(Page {
                    content_type: detect_content_type(&path).into(),
                    contents,
                });
            } else {
                tracing::warn!("failed to load {:?}: {}", path, file.unwrap_err());
            }
        }
        return Err(LoadError(format!("failed to load any of ")));
    }

    fn new() -> Result<Self, LoadError> {
        let mut svr = Server {
            template: "".into(),
        };
        svr.template = svr.read_file(path::Path::new("templates/main.html"))?;
        Ok(svr)
    }
}

async fn handle_page(
    server: Arc<RwLock<Server>>,
    path: &str,
) -> Result<Response, (StatusCode, String)> {
    let server = server.read().await;

    let page_input = match server.read_request_file(path::Path::new(path)) {
        Ok(i) => i,
        Err(e) => return Err((StatusCode::NOT_FOUND, format!("Failed to read file: {}", e))),
    };
    let page = server.template.replace("{{content}}", &page_input.contents);
    let mut response = page.into_response();
    response.headers_mut().append(
        "Content-Type",
        HeaderValue::from_str(&page_input.content_type).expect("wtf what do you want from me"),
    );
    return Ok(response);
}

async fn root_route(
    State(server): State<Arc<RwLock<Server>>>,
) -> Result<Response, (StatusCode, String)> {
    return handle_page(server, "index").await;
}

async fn path_route(
    State(server): State<Arc<RwLock<Server>>>,
    Path(path): Path<String>,
) -> Result<Response, (StatusCode, String)> {
    return handle_page(server, &path).await;
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    // tracing_subscriber::fmt()
    //     .with_target(false)
    //     .compact()
    //     .init();

    let server = match Server::new() {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to start server: {}", e);
            return Err(());
        }
    };
    let server = Arc::new(RwLock::new(server));

    let router = Router::new()
        .route("/", get(root_route))
        .route("/*path", get(path_route))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    async fn handler_404() -> impl IntoResponse {
        return (StatusCode::NOT_FOUND, "404 Not Found");
    }

    let app = router.fallback(handler_404).with_state(server);

    axum::Server::bind(&SocketAddr::from(([0, 0, 0, 0], 2137)))
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
