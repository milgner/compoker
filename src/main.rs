use actix_files::{Files, NamedFile};
use actix_web::{App, HttpServer};
use actix_web::dev::{ServiceRequest, ServiceResponse};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(Files::new("/", "./public")
            .prefer_utf8(true)
            .index_file("index.html")
            // for SPA behaviour: unknown/dynamic paths will be resolved through app routing mechanism
            .default_handler(|req: ServiceRequest| {
                let (http_req, _payload) = req.into_parts();

                async {
                    let response = NamedFile::open("./public/index.html")?.into_response(&http_req)?;
                    Ok(ServiceResponse::new(http_req, response))
                }
            }))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
