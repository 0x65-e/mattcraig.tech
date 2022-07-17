use std::path::{PathBuf, Path};
use std::borrow::Borrow;
use worker::*;
use crate::kv::{KvStore, GetOptionsBuilder};

mod utils;

async fn retrieve_file_from_static_store(path: &str, env: &Env) -> Result<Response> {
    let static_store: KvStore = env.kv("STATIC")?;

    let mut pathbuf: PathBuf = PathBuf::from(String::from(path));
    // Default to index.html if the page is not specified within the directory
    if pathbuf.ends_with("/") {
        pathbuf.push("index.html"); // TODO: this isn't working 100% - see /resume/ for an example
    }
    // Default to .html if there is no extension
    if pathbuf.extension() == None {
        pathbuf.set_extension("html");
    }
    // Remove leading slash to index into kv store
    let path : &Path = pathbuf.strip_prefix("/").unwrap_or(pathbuf.as_path());

    // Set Content-Type header based on file extension (naive)
    let mut headers : Headers = Headers::new();
    let content_type: &str = match path.extension().unwrap().to_str() { // use unwrap() since we checked for extension() == None in the PathBuf
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "text/javascript",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("jpg") => "image/jpeg",
        Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("wav") => "audio/wav",
        Some("mp3") => "audio/mpeg",
        Some("mpeg") => "audio/mpeg",
        Some("mp4") => "video/mp4",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("csv") => "text/csv",
        Some("txt") => "text/plain",
        Some("pdf") => "application/pdf",
        Some("zip") => "application/zip",
        Some("7z") => "application/x-7z-compressed",
        Some(_) => "text/plain", // Default to plain text for any other extension
        None => {
            utils::log_invalid_filename(path.to_string_lossy().borrow());
            return Response::error(utils::create_error_response("Bad Request", "400 Bad Request", "Sorry, non-unicode characters are not permitted."), 400); // Non-Unicode characters are not supported in filename keys
        },
    };
    headers.set("Content-Type", content_type)?; // Content-Type is always a valid header, so ? should never panic

    if path.to_str() == None {
        utils::log_invalid_filename(path.to_string_lossy().borrow());
        return Response::error(utils::create_error_response("Bad Request", "400 Bad Request", "Sorry, non-unicode characters are not permitted."), 400); // Non-Unicode characters are not supported in filename keys
    }
    let result: GetOptionsBuilder = static_store.get(path.to_str().unwrap());
    return match result.bytes().await? {
        Some(bytes) => Ok(Response::from_bytes(bytes)?.with_headers(headers)),
        None => {
            utils::log_not_present_error("STATIC", path.to_str().unwrap());
            Response::error(utils::create_error_response("Not Found", "404 Not Found", "Oops, looks like we weren't able to find the webpage you were looking for."), 404)
        }
    };
}

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
                return match retrieve_file_from_static_store(path, &ctx.env).await {
                    Ok(response) => Ok(response),
                    Err(e) => {
                        utils::log_generic_error(path, &e.to_string());
                        // Generic error message
                        Response::error(utils::create_error_response("Bad Request", "500 Internal Server Error", "Sorry, something went wrong and we're unable to handle your request."), 500)
                    }
                };
            } else {
                // No path parameter - bad client request
                return Response::error(utils::create_error_response("Bad Request", "400 Bad Request", "Looks like that's not a valid path on this server!"), 400);
            }
        })
        .run(req, env)
        .await
}
