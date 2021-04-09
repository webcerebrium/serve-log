use actix_web::web::{BytesMut, Payload};
use actix_web::{error, Error, HttpRequest};
use futures::StreamExt; // provides implementation for web::Payload.next()

mod server {
    pub fn get_bind_address() -> String {
        std::env::var("BIND_ADDR")
            .ok()
            .unwrap_or("0.0.0.0:5000".to_owned())
    }

    pub fn get_web_root() -> String {
        std::env::var("WEB_ROOT").ok().unwrap_or("./".to_owned())
    }

    pub fn get_cors_factory() -> actix_cors::CorsFactory {
        actix_cors::Cors::new()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::ACCEPT,
            ])
            .max_age(36000)
            .finish()
    }
}

pub async fn get_payload_str(mut payload: Payload) -> Result<String, Error> {
    // reading payload: pretty low level works for error handler
    let mut bytes = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (bytes.len() + chunk.len()) > 4 * 16386 {
            return Err(error::ErrorBadRequest("overflow"));
        }
        bytes.extend_from_slice(&chunk);
    }
    let body_str = std::str::from_utf8(&bytes)?;
    return Ok(String::from(body_str));
}

async fn index(req: HttpRequest, payload: Payload) -> Result<actix_files::NamedFile, Error> {
    println!("\n{} {:#?} {}", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"), req.method(), req.path());
    let headers = req.headers().clone();
    for (k, v) in headers.iter() {
        println!("  H {:?}: {:?}", k, v);
    }
    if req.query_string().as_bytes().len() > 0 {
        println!("  Q {:#?}", req.query_string());
    }
    match get_payload_str(payload).await {
        Ok(payload) => {
            if payload.len() > 0 {
                println!("PAYLOAD:\n---\n{}\n---", payload.as_str());
            }
        }
        Err(errp) => println!("PAYLOAD ERROR: {:?}", errp),
    }
    let src: String = server::get_web_root() + "/" + String::from("index.txt").as_str();
    let fallback: String = server::get_web_root() + "/index.txt";
    let found: bool = std::path::Path::new(&src).exists();
    let file = actix_files::NamedFile::open(if found { &src } else { &fallback })?;
    Ok(file.use_last_modified(true))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    println!(
        "Starting server at http://{}/, static files root at {}",
        server::get_bind_address(),
        server::get_web_root()
    );
    actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(server::get_cors_factory())
            .route("/.*", actix_web::web::post().to(index))
            .route("/.*", actix_web::web::get().to(index))
            .route("/", actix_web::web::post().to(index))
            .route("/", actix_web::web::get().to(index))
    })
    // .workers(1)
    .bind(server::get_bind_address())?
    .run()
    .await
}
