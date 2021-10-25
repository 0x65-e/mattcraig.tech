use serde_json::json;
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

fn log_bad_format_error(kv: &String, key: &String) {
    console_log!(
        "{} - [{}], problem interpreting key \"{}\" as file",
        Date::now().to_string(),
        kv,
        key
    );
}

fn log_not_present_error(kv: &String, key: &String) {
    console_log!(
        "{} - [{}], key \"{}\" not present in store",
        Date::now().to_string(),
        kv,
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
                    let mut path: String = path.clone();
                    // Default to index.html if the page is not specified within the directory
                    if path.ends_with("/") {
                        path.push_str("index.html");
                    }
                    // Remove leading /
                    let path = path.strip_prefix('/').unwrap_or(&path);
                    match static_store.get(&path).await {
                        Ok(result) => match result {
                            Some(file) =>  return Response::from_html(file.as_string()),
                            None => {
                                log_not_present_error(&String::from("STATIC"), &String::from(path));
                                return Response::error("Not Found", 404);
                            }
                        },
                        Err(_) => {
                            return Response::error("Internal Service Error", 500); // Unable to reach KV store
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
