use rusqlite::{Connection, OptionalExtension, Result};

pub fn db_open() -> Result<Connection> {
    let conn = Connection::open("db/index.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS volumes(
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        regenerate_epub INTEGER DEFAULT 0 CHECK(regenerate_epub IN (0, 1)),
        UNIQUE(name)
    )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS chapters(
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        uri TEXT NOT NULL,
        volumeid INTEGER,
        data_id INTEGER,
        regenerate_epub INTEGER DEFAULT 0 CHECK(regenerate_epub IN (0, 1)),
        FOREIGN KEY(data_id) REFERENCES raw_data(id),
        FOREIGN KEY(volumeid) REFERENCES volume(id),
        UNIQUE(name, uri, volumeid)
    )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS raw_data(
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        chapter_id INTEGER,
        data TEXT,
        FOREIGN KEY(chapter_id) REFERENCES chapters(id),
        UNIQUE(chapter_id)
    )",
        (),
    )?;

    Ok(conn)
}

pub fn db_add_volume(db_conn: &Connection, name: &String) -> Result<usize> {
    db_conn
        .prepare("INSERT OR IGNORE INTO volumes(name) values(?1)")?
        .execute([name])?;
    Ok(
        db_conn.query_row("SELECT id FROM volumes WHERE name = ?1", [name], |row| {
            row.get(0)
        })?,
    )
}

pub fn db_add_chapter(
    db_conn: &Connection,
    name: String,
    uri: String,
    volume: usize,
) -> Result<()> {
    db_conn
        .prepare("INSERT OR IGNORE INTO chapters(name, uri, volumeid) values(?1, ?2, ?3)")?
        .execute((name, uri, volume))?;
    Ok(())
}

pub fn add_chapter_data(db_conn: &Connection, chapter_id: usize, data: &String) -> Result<()> {
    let existing_data = match db_conn
        .query_row(
            "SELECT data FROM raw_data WHERE chapter_id = ?1",
            [chapter_id],
            |row| row.get(0),
        )
        .optional()?
    {
        Some(data) => data,
        None => "".to_string(),
    };

    db_conn
        .prepare("INSERT OR REPLACE INTO raw_data(data, chapter_id) values(?1, ?2)")?
        .execute((data, chapter_id))?;
    let data_id: usize =
        db_conn.query_row("SELECT id FROM raw_data WHERE data = ?1", [data], |row| {
            row.get(0)
        })?;
    db_conn
        .prepare("UPDATE chapters SET data_id = ?1, regenerate_epub = ?2 WHERE id = ?3")?
        .execute([data_id, !existing_data.eq(data) as usize, chapter_id])?;
    Ok(())
}
