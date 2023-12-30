use anyhow::{Ok, Result};
use axum::{
    http::{HeaderMap, Uri, header, Request},
    response::{IntoResponse, Response},
    serve, Router, body::Body,
};
use tower_http::services::ServeFile;
use urlencoding::decode;
use std::{env, fs, net::{SocketAddr, Ipv4Addr}};
use tokio::net;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 0);
    let listener = net::TcpListener::bind(&addr).await?;

    let app = Router::new().fallback(handler);
    println!("Rustatic starting at http://{}", listener.local_addr().unwrap());
    serve(listener, app).await?;
    Ok(())
}

async fn handler(uri: Uri, headers: HeaderMap) -> Response {
    let uri = uri.to_string();
    let decoded_uri = decode(&uri).unwrap();
    let req_path = &decoded_uri.to_string()[1..]; // removing the / from beginning to make it a non-absolute path

    let mut target_path = env::current_dir().expect("Cannot get the current directory!");
    target_path.push(req_path);

    if !target_path.exists() {
        let response_body = "<html><body>Unable to access the specified path</body></html>";
        return (
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CONTENT_LENGTH, &response_body.len().to_string())
            ],
            response_body
        ).into_response()
    }

    let metadata = fs::metadata(&target_path).unwrap();
    if metadata.is_dir() {
        let mut response_body = String::new();
        response_body.push_str("<html><body><h1>Directory listing for /</h1><hr><ul>");

        for entry in fs::read_dir(&target_path).unwrap() {
            let entry = entry.unwrap();
            let is_dir = entry.file_type().unwrap().is_dir();
            let entry_name = entry.file_name();    
            let entry_name = entry_name.to_string_lossy()[0..entry_name.len()].to_owned();
            let entry_path = entry_name + if is_dir { "/" } else { "" };

            response_body.push_str(
                &format!(
                    "<li><a href='{}'>{}</a></li>",
                    entry_path,
                    entry_path
                )
            )
        }

        response_body.push_str("</ul><hr></body></html>");
        return (
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CONTENT_LENGTH, &response_body.len().to_string())
            ],
            response_body
        ).into_response()
    }

    let mut req = Request::new(Body::empty());
    *req.headers_mut() = headers;
    ServeFile::new(&target_path).try_call(req).await.unwrap().into_response()
    // TODO: Implement own file serving logic
}
