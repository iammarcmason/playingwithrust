use actix_web::{web, HttpResponse, Result};
use rusqlite::Connection;
use tera::Tera;
use crate::models::Content;

pub async fn add_content(content: web::Form<Content>, _tera: web::Data<Tera>) -> Result<HttpResponse> {
    let conn = match Connection::open("rocket.db") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open database connection: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    let topic_exists = !content.topic.is_empty(); // Check if topic is provided

    // Print the received content for debugging
    println!("Received content: {:?}", content);

    if topic_exists {
        let topic_id = match conn.query_row(
            "SELECT topic_id FROM topic WHERE topic_name = ?",
            &[&content.topic],
            |row| row.get::<usize, i64>(0), // Retrieve the topic ID
        ) {
            Ok(id) => Some(id),
            Err(_) => None, // Topic doesn't exist
        };

        // If the topic doesn't exist, add it
        if topic_id.is_none() {
            if let Err(err) = conn.execute(
                "INSERT INTO topic (topic_name) VALUES (?)",
                &[&content.topic],
            ) {
                eprintln!("Failed to add topic: {}", err);
                let message = format!("Failed to add topic: {}", err);
                return Ok(HttpResponse::InternalServerError().body(message));
            }
        }

        // Retrieve the topic ID again (whether it's newly added or already existed)
        let topic_id = match conn.query_row(
            "SELECT topic_id FROM topic WHERE topic_name = ?",
            &[&content.topic],
            |row| row.get::<usize, i64>(0),
        ) {
            Ok(id) => id,
            Err(_) => {
                eprintln!("Failed to fetch topic ID");
                return Ok(HttpResponse::InternalServerError().finish());
            }
        };

        // Insert into 'sub_topic' table
        if let Err(err) = conn.execute(
            "INSERT INTO sub_topic (sub_topic_name, topic_id) VALUES (?, ?)",
            &[&content.subtopic, &topic_id as &dyn rusqlite::types::ToSql],
        ) {
            eprintln!("Failed to add subtopic: {}", err);
            let message = format!("Failed to add subtopic: {}", err);
            return Ok(HttpResponse::InternalServerError().body(message));
        }

        // Insert into 'content' table
        if let Err(err) = conn.execute(
            "INSERT INTO content (content_text, sub_topic_id) VALUES (?, (SELECT sub_topic_id FROM sub_topic WHERE sub_topic_name = ?))",
            &[&content.content, &content.subtopic],
        ) {
            let message = format!("Failed to add content: {}", err);
            return Ok(HttpResponse::Ok().body(message));
        }

        let success_message = "Data added successfully!";
        println!("{}", success_message);
        Ok(HttpResponse::Ok().body(success_message))
    } else {
        let message = "Topic is required".to_string();
        Ok(HttpResponse::Ok().body(message))
    }
}
