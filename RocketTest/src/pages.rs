use actix_web::{http, web, HttpResponse, Responder, Result};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use tera::{Context, Tera};


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

    let mut stmt = match conn.prepare(
        "
        SELECT topic_name, sub_topic_name
        FROM topic
        LEFT JOIN sub_topic ON topic.topic_id = sub_topic.topic_id
    ",
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to prepare SQL statement: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    let rows = match stmt.query_map([], |row| {
        Ok((row.get::<usize, String>(0)?, row.get::<usize, String>(1)?))
    }) {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch topics and subtopics: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    let mut topic_subtopics: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let (topic, subtopic) = match row {
            Ok((topic, subtopic)) => (topic, subtopic),
            Err(e) => {
                eprintln!("Error reading row: {}", e);
                continue;
            }
        };
        topic_subtopics
            .entry(topic)
            .or_insert_with(Vec::new)
            .push(subtopic);
    }

    let topics: Vec<String> = topic_subtopics.keys().cloned().collect();

    let mut context = Context::new();
    context.insert("topics", &topics);
    context.insert("subtopics", &topic_subtopics);

    match tera.render("index.html", &context) {
        Ok(rendered) => Ok(HttpResponse::Ok().content_type("text/html").body(rendered)),
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}


pub 
async fn get_topic_page(tera: web::Data<Tera>, info: web::Path<String>) -> impl Responder {
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




pub async fn addrow(tera: web::Data<Tera>) -> Result<HttpResponse> {
    let conn = match Connection::open("rocket.db") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open database connection: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    // Fetch existing topics from the 'topic' table
    let topics = match conn.prepare("SELECT topic_name FROM topic") {
        Ok(mut stmt) => {
            let topic_iter = stmt.query_map([], |row| row.get(0));
            match topic_iter {
                Ok(topics) => topics.map(|t| t.unwrap()).collect::<Vec<String>>(),
                Err(e) => {
                    eprintln!("Failed to fetch topics: {}", e);
                    return Ok(HttpResponse::InternalServerError().finish());
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to prepare SQL statement: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    let success_message = "Data added successfully!";
    let mut context = Context::new();
    context.insert("message", success_message);
    context.insert("topics", &topics);

    match tera.render("addrow.html", &context) {
        Ok(rendered) => Ok(HttpResponse::Ok()
            .content_type("text/html")
            .append_header((http::header::CACHE_CONTROL, "no-cache"))
            .body(rendered)),
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}