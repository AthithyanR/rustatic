use anyhow::{Ok, Result};
use axum::{
    http::{HeaderMap, Uri},
    response::{Html, IntoResponse},
    serve, Router,
};
use std::{env, fs};
use tokio::net;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().fallback(handler);
    let listener = net::TcpListener::bind("0.0.0.0:3000").await?;
    serve(listener, app).await?;
    Ok(())
}

async fn handler(uri: Uri) -> impl IntoResponse {
    let req_path = &uri.to_string()[1..]; // removing the / from beginning to make it a non-absolute path

    let mut target_path = env::current_dir().expect("Cannot get the current directory!");
    target_path.push(req_path);

    let mut headers = HeaderMap::new();
    headers.insert("content-type", "text/html".parse().unwrap());

    if !target_path.exists() {
        return Html("<html><body>Unable to access the specified path</body></html>".to_string());
    }

    let metadata = fs::metadata(&target_path).unwrap();
    if metadata.is_dir() {
        let mut response = String::new();
        response.push_str("<html><body><h1>Directory listing for /</h1><hr><ul>");

        for entry in fs::read_dir(&target_path).unwrap() {
            let entry = entry.unwrap();
            let file_name = entry.file_name();    
            let file_name = &file_name.to_string_lossy()[0..file_name.len()];

            response.push_str(
                &format!(
                    "<li>
                        <a href='/{}'>{}</a>
                    </li>",
                    file_name,
                    file_name,
                )
            )
        }

        response.push_str("</ul><hr></body></html>");
        return Html(response);
    }

   Html("<html><body>Is a file</body></html>".to_string())
}
