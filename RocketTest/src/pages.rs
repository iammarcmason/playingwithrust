//use actix_web::{http, web, HttpResponse, Responder, Result};
use actix_web::{web, HttpResponse, Responder, Result};
use rusqlite::{params, Connection};
use tera::{Context, Tera};

use crate::getter;

pub async fn handle_404(tera: web::Data<Tera>) -> Result<HttpResponse> {
    let mut context = Context::new();
    context.insert("message", "Page not found");

    match tera.render("404.html", &context) {
        Ok(rendered) => Ok(HttpResponse::NotFound()
            .content_type("text/html")
            .body(rendered)),
        Err(e) => {
            eprintln!("Template render error: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

pub async fn index(tera: web::Data<Tera>) -> Result<HttpResponse> {
    let conn = match Connection::open("rocket.db") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open database connection: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    let (sorted_topics, sorted_topic_subtopics) = match getter::get_topics_and_subtopics(&conn) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to fetch topics and subtopics: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    let mut context = Context::new();
    context.insert("topics", &sorted_topics);
    context.insert("subtopics", &sorted_topic_subtopics);

    match tera.render("index.html", &context) {
        Ok(rendered) => Ok(HttpResponse::Ok().content_type("text/html").body(rendered)),
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

pub async fn get_topic_page(tera: web::Data<Tera>, info: web::Path<String>) -> impl Responder {
    let topic = info.into_inner(); // Extract the topic name from the URL

    // Open a connection to your SQLite database
    let conn = match Connection::open("rocket.db") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Fetch subtopics for the given topic from the database
    let mut stmt = match conn.prepare(
        "
        SELECT sub_topic_name
        FROM sub_topic
        INNER JOIN topic ON sub_topic.topic_id = topic.topic_id
        WHERE topic.topic_name = ?
    ",
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to prepare SQL statement: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let subtopics: Vec<String> = match stmt.query_map(params![&topic], |row| row.get(0)) {
        Ok(subtopic_iter) => {
            subtopic_iter
                .filter_map(|s| s.ok()) // Filter out potential errors
                .collect()
        }
        Err(e) => {
            eprintln!("Failed to fetch subtopics: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut context = Context::new();
    context.insert("topic", &topic);
    context.insert("subtopics", &subtopics);

    match tera.render("topic_page.html", &context) {
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// pub async fn addrow(tera: web::Data<Tera>) -> Result<HttpResponse> {
//     let conn = match Connection::open("rocket.db") {
//         Ok(c) => c,
//         Err(e) => {
//             eprintln!("Failed to open database connection: {}", e);
//             return Ok(HttpResponse::InternalServerError().finish());
//         }
//     };

//     // Fetch existing topics from the 'topic' table
//     let topics = match conn.prepare("SELECT topic_name FROM topic") {
//         Ok(mut stmt) => {
//             let topic_iter = stmt.query_map([], |row| row.get(0));
//             match topic_iter {
//                 Ok(topics) => topics.map(|t| t.unwrap()).collect::<Vec<String>>(),
//                 Err(e) => {
//                     eprintln!("Failed to fetch topics: {}", e);
//                     return Ok(HttpResponse::InternalServerError().finish());
//                 }
//             }
//         }
//         Err(e) => {
//             eprintln!("Failed to prepare SQL statement: {}", e);
//             return Ok(HttpResponse::InternalServerError().finish());
//         }
//     };

//     let success_message = "Data added successfully!";
//     let mut context = Context::new();
//     context.insert("message", success_message);
//     context.insert("topics", &topics);

//     match tera.render("addrow.html", &context) {
//         Ok(rendered) => Ok(HttpResponse::Ok()
//             .content_type("text/html")
//             .append_header((http::header::CACHE_CONTROL, "no-cache"))
//             .body(rendered)),
//         Err(e) => {
//             eprintln!("Template rendering error: {}", e);
//             Ok(HttpResponse::InternalServerError().finish())
//         }
//     }
// }
