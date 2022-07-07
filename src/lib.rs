use std::path::*;
use std::borrow::Borrow;
use worker::*;

mod utils;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get_async("/*path", |_, ctx| async move {
            if let Some(path) = ctx.param("path") {
                if let Ok(static_store) = ctx.kv("STATIC") {
                    let mut pathbuf: PathBuf = PathBuf::from(String::from(path));
                    // Default to index.html if the page is not specified within the directory
                    if pathbuf.ends_with("/") {
                        pathbuf.push("index.html");
                    }
                    // Add .html if there is no extension
                    if pathbuf.extension() == None {
                        pathbuf.push(".html");
                    }
                    // Remove leading /
                    let path : &Path = pathbuf.strip_prefix("/").unwrap_or(pathbuf.as_path()); // may need & on pathbuf?
                    // Set Content-Type header based on file extension (naive)
                    let mut headers : Headers = Headers::new();
                    let content_type = match path.extension().unwrap().to_str() { // use unwrap() since we checked for extension() == None in the PathBuf
                        Some("html") => "text/html",
                        Some("css") => "text/css",
                        Some("js") => "text/javascript",
                        Some("json") => "application/json",
                        Some("svg") => "image/svg+xml",
                        Some("jpg") => "image/jpeg",
                        Some("woff") => "font/woff",
                        Some("woff2") => "font/woff2",
                        Some("ttf") => "font/ttf",
                        Some(_) => "text/plain", // Default to plain text for any other extension
                        None => {
                            utils::log_invalid_filename(path.to_string_lossy().borrow());
                            return Response::error("Bad Request", 400); // Non-Unicode characters are not supported
                        },
                    };
                    headers.set("Content-Type", content_type)?; // Fine to use ? here since Content-Type is always a valid header
                    match path.to_str() {
                        Some(path) => {
                            let result = static_store.get(path);
                            match result.bytes().await {
                                Ok(bytes) => match bytes {
                                    Some(bytes) => return Ok(Response::from_bytes(bytes)?.with_headers(headers)),
                                    None => {
                                        utils::log_not_present_error("STATIC", path);
                                        return Response::error("Not Found", 404);
                                    },
                                },
                                Err(e) => return Response::error("Internal Server Error", 500), //TODO: Distinguish between different types of KvError
                            }
                        },
                        None => { 
                            utils::log_invalid_filename(path.to_string_lossy().borrow());
                            return Response::error("Bad Request", 400); // Non-Unicode characters are not supported
                        }
                    }
                } else {
                    return Response::error("Internal Server Error", 500); // KV store doesn't exist
                }
            } else {
                return Response::error("Bad Request", 400);
            }
        })
        .run(req, env)
        .await
}
