use rusqlite::{Connection, Result as SqlResult};


pub fn create_tables(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS topic (
            topic_id INTEGER PRIMARY KEY,
            topic_name TEXT NOT NULL UNIQUE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sub_topic (
            sub_topic_id INTEGER PRIMARY KEY,
            sub_topic_name TEXT NOT NULL UNIQUE,
            topic_id INTEGER NOT NULL,
            FOREIGN KEY (topic_id) REFERENCES topic(topic_id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS content (
            content_id INTEGER PRIMARY KEY,
            content_text TEXT NOT NULL,
            sub_topic_id INTEGER NOT NULL,
            FOREIGN KEY (sub_topic_id) REFERENCES sub_topic(sub_topic_id)
        )",
        [],
    )?;

    Ok(())
}