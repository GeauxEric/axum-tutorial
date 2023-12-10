use axum::body::Body;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Router;
use axum::routing::get;
use mime_guess::mime;

/// 1. launch Axum server
/// 1.1 add the dependency
/// 1.2 compile it and run
/// 2. handle request to list directories
/// 2.1 working directory, "/", get, list_wd
/// 2.2 sub directory
/// 3. handle request to show file content
/// 3.1 determine MIME type
/// 3.2 return the file content
///
/// TODO:
/// 1. logging and tracing requests
/// 2. command line argument, e.g. port
/// 3. streaming file content
///
/// To understand more about axum:
/// 1. youtube: https://youtu.be/Wnb_n5YktO8?si=hjVeUfJizLvDnflM
/// 2. axum project examples: https://github.com/tokio-rs/axum/tree/main/examples
#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/", get(list_wd))
        .route("/*path", get(handle_path));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_path(Path(path): Path<String>) -> Response {
    let fs_path_str = format!("./{}", path);

    let path = std::path::Path::new(&fs_path_str);

    // Use Tokio to asynchronously retrieve metadata for the path
    let metadata = tokio::fs::metadata(path).await.unwrap();

    if metadata.is_dir() {
        return list_dir(path).await.into_response();
    } else if metadata.is_file() || metadata.is_symlink() {
        let guess = mime_guess::from_path(path).first();
        let mime_type = guess.unwrap_or(mime::APPLICATION_OCTET_STREAM);
        let bytes = tokio::fs::read(path).await.unwrap();
        return Response::builder()
            .header(axum::http::header::CONTENT_TYPE, mime_type.to_string())
            .status(StatusCode::OK)
            .body(Body::from(bytes))
            .unwrap();
    } else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "unhandled type",
        ).into_response();
    }
}

/// path can be "." or "./foo/qoo"
async fn list_dir(dir_path: &std::path::Path) -> Html<String> {
    let url_dir_path = dir_path.strip_prefix(".").unwrap().as_os_str().to_str().unwrap();
    let url_dir_path = format!("{}/", url_dir_path);

    // Read the directory contents asynchronously
    let mut entries = tokio::fs::read_dir(dir_path).await.unwrap();

    // Create an HTML string
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<title>Directory Listing</title>\n</head>\n<body>\n");
    let header = format!("<h1>Directory Listing for {}</h1>\n<ul>\n", url_dir_path);
    html.push_str(&header);

    // Iterate over directory entries and add them to the HTML
    while let Some(entry) = entries.next_entry().await.unwrap() {
        let entry_path = entry.path();
        let entry_name = entry_path.file_name().unwrap_or_default().to_string_lossy();
        let meta = tokio::fs::metadata(&entry_path).await.unwrap();
        let link = if meta.is_dir() { format!("{}/", entry_name) } else { entry_name.to_string() };
        let link = format!("<a href={}>{}</a>", link, entry_name);
        html.push_str(&format!("<li>{}</li>\n", link));
    }

    html.push_str("</ul>\n</body>\n</html>");
    return Html(html);
}

/// List the content of working directory
/// Returns html document as a string
async fn list_wd() -> Html<String> {
    return list_dir(&std::path::Path::new(".")).await;
}