use actix_web::{web, App, HttpServer};
use actix_files as fs;
use rusqlite::Connection;
use tera::Tera;

mod getter;
mod models;
mod pages;
mod setter;
mod sql_func;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to initialize Tera: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to initialize Tera",
            ));
        }
    };

    // Open or create the SQLite database file
    let conn = match Connection::open("rocket.db") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open database connection: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to open database connection",
            ));
        }
    };

    // Create tables if they don't exist
    if let Err(err) = sql_func::create_tables(&conn) {
        eprintln!("Failed to create tables: {}", err);
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .service(
                web::scope("")
                    .route("/", web::get().to(pages::index)) // Handle root path
                    .route("/addrow", web::get().to(pages::addrow))
                    .route(
                        "/topics/{topic}/{subtopic}",
                        web::get().to(getter::get_content),
                    )
                    .route("/add", web::post().to(setter::add_content)) // Add route for handling form submission
                    .service(
                        web::resource("/topics/{topic}")
                            .route(web::get().to(pages::get_topic_page)),
                    )
                    .service(fs::Files::new("/static", "static").show_files_listing())
            
                    
            )
            .default_service(web::route().to(pages::handle_404))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
