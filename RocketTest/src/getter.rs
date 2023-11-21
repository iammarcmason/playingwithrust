use actix_web::{web, HttpResponse, Responder};
use pulldown_cmark::{html, Parser};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::fs;
use tera::{Context, Tera};

pub async fn get_content(
    info: web::Path<(String, String)>,
    tera: web::Data<Tera>,
) -> impl Responder {
    let topic = info.0.clone();
    let subtopic = info.1.clone();
    let db_path = "rocket.db";

    let actual_db_path = match fs::canonicalize(db_path) {
        Ok(path) => {
            println!("Running Getter, trying database path: {:?}", path);
            path
        }
        Err(e) => {
            eprintln!("Failed to retrieve database path: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Open a connection to your SQLite database
    let conn = match Connection::open(&actual_db_path) {
        Ok(c) => {
            println!(
                "Successfully connected to database: {}",
                actual_db_path.display()
            );
            c
        }
        Err(e) => {
            eprintln!(
                "Failed to open database connection to {}: {}",
                actual_db_path.display(),
                e
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Prepare and execute the SQL query
    let mut stmt = match conn.prepare(
        "
        SELECT content_text
        FROM content
        INNER JOIN sub_topic ON content.sub_topic_id = sub_topic.sub_topic_id
        INNER JOIN topic ON sub_topic.topic_id = topic.topic_id
        WHERE topic.topic_name = ?1 AND sub_topic.sub_topic_name = ?2
    ",
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to prepare SQL statement: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Execute the query with the provided topic and subtopic
    let result = match stmt.query_row(params![&topic, &subtopic], |row| {
        row.get::<usize, String>(0)
    }) {
        Ok(r) => Ok(r),
        Err(e) => {
            eprintln!("Query execution error: {}", e);
            Err("Content not found".to_string()) // Return an Err containing a message
        }
    };

    // Process the result
    let content = match result {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Content retrieval error: {}", err);
            err
        }
    };

    // Parse Markdown content to HTML
    let parser = Parser::new(&content);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let mut context = Context::new();
    context.insert("content", &html_output);

    match tera.render("content.html", &context) {
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub fn get_topics_and_subtopics(
    conn: &Connection,
) -> Result<(Vec<String>, HashMap<String, Vec<String>>), rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT topic_name, sub_topic_name
        FROM topic
        LEFT JOIN sub_topic ON topic.topic_id = sub_topic.topic_id",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((row.get::<usize, String>(0)?, row.get::<usize, String>(1)?))
    })?;

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

    let mut sorted_topics = topic_subtopics.keys().cloned().collect::<Vec<_>>();
    sorted_topics.sort(); // Sort topics alphabetically

    Ok((sorted_topics, topic_subtopics))
}
