use std::path::*;
use std::borrow::Borrow;
use worker::*;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

fn log_bad_format_error(kv: &str, key: &str) {
    console_log!(
        "{} - [{}], problem interpreting key \"{}\" as base64 encoded file",
        Date::now().to_string(),
        kv,
        key
    );
}

fn log_not_present_error(kv: &str, key: &str) {
    console_log!(
        "{} - [{}], key \"{}\" not present in store",
        Date::now().to_string(),
        kv,
        key
    );
}

fn log_invalid_filename(key: &str) {
    console_log!(
        "{} - [{}], requested page \"{}\" uses non-UTF-8 characters",
        Date::now().to_string(),
        key,
        key
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    log_request(&req);

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
                    let mut base64encoded : bool = false; // Some file formats are raw data, which are stored as base64 encoded files in the KV
                    match path.extension().unwrap().to_str() { // use unwrap() since we checked for extension() == None in the PathBuf
                        Some("html") => {base64encoded = false; headers.set("Content-Type", "text/html")?;}, // Fine to use ? here since Content-Type is always a valid header
                        Some("css") => {base64encoded = false; headers.set("Content-Type", "text/css")?;},
                        Some("js") => {base64encoded = false; headers.set("Content-Type", "text/javascript")?;},
                        Some("json") => {base64encoded = false; headers.set("Content-Type", "application/json")?;},
                        Some("svg") => {base64encoded = false; headers.set("Content-Type", "image/svg+xml")?;},
                        Some("jpg") => {base64encoded = true; headers.set("Content-Type", "image/jpeg")?;},
                        Some("woff") => {base64encoded = true; headers.set("Content-Type", "font/woff")?;},
                        Some("woff2") => {base64encoded = true; headers.set("Content-Type", "font/woff2")?;},
                        Some("ttf") => {base64encoded = true; headers.set("Content-Type", "font/ttf")?;},
                        Some(_) => {base64encoded = false; headers.set("Content-Type", "text/plain")?;}, // Default to plain text for any other extension
                        None => {
                            log_invalid_filename(path.to_string_lossy().borrow());
                            return Response::error("Bad Request", 400); // Non-Unicode characters are not supported
                        },
                    };
                    match path.to_str() {
                        Some(path) => {
                            match static_store.get(path).await {
                                Ok(result) => match result {
                                    Some(file) => {
                                        if base64encoded { 
                                            // Decode binary file formats into raw bytes
                                            match base64::decode(file.as_string()) {
                                                Ok(bytes) => return Ok(Response::from_bytes(bytes)?.with_headers(headers)),
                                                Err(_) => {
                                                    log_bad_format_error("STATIC", path);
                                                    return Response::error("Not Found", 404);
                                                }
                                            }
                                        } else { 
                                            return Ok(Response::from_bytes(file.as_bytes().to_vec())?.with_headers(headers));
                                        };
                                        
                                    },
                                    None => {
                                        log_not_present_error("STATIC", path);
                                        return Response::error("Not Found", 404);
                                    }
                                },
                                Err(_) => return Response::error("Internal Server Error", 500), // Unable to reach KV store
                            }
                        },
                        None => { 
                            log_invalid_filename(path.to_string_lossy().borrow());
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
