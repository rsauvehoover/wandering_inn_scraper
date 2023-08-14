use rusqlite::{Connection, OptionalExtension, Params, Result};
use std::path::Path;

pub struct Chapter {
    pub id: usize,
    pub name: String,
    pub uri: String,
    pub volumeid: usize,
    pub data_id: usize,
}

pub struct Volume {
    pub id: usize,
    pub name: String,
}

pub fn open() -> Result<Connection> {
    std::fs::create_dir_all(Path::new("db")).unwrap();

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
        FOREIGN KEY(volumeid) REFERENCES volumes(id),
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

pub fn add_volume(db_conn: &Connection, name: &String) -> Result<usize> {
    db_conn
        .prepare("INSERT OR IGNORE INTO volumes(name) values(?1)")?
        .execute([name])?;
    Ok(
        db_conn.query_row("SELECT id FROM volumes WHERE name = ?1", [name], |row| {
            row.get(0)
        })?,
    )
}

pub fn add_chapter(db_conn: &Connection, name: String, uri: String, volume: usize) -> Result<()> {
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

    let regenerate = !existing_data.eq(data);
    let data_id: usize =
        db_conn.query_row("SELECT id FROM raw_data WHERE data = ?1", [data], |row| {
            row.get(0)
        })?;
    db_conn
        .prepare("UPDATE chapters SET data_id = ?1, regenerate_epub = ?2 WHERE id = ?3")?
        .execute([data_id, regenerate as usize, chapter_id])?;

    let volume_id: usize = db_conn.query_row(
        "SELECT volumeid FROM chapters WHERE id = ?1",
        [chapter_id],
        |row| row.get(0),
    )?;
    db_conn
        .prepare("UPDATE volumes SET regenerate_epub = ?1 WHERE id = ?2")?
        .execute([regenerate as usize, volume_id])?;
    Ok(())
}

fn chapter_query_helper<P>(db_conn: &Connection, sql: &str, params: P) -> Result<Vec<Chapter>>
where
    P: Params,
{
    let mut stmt = db_conn.prepare(sql)?;
    let res = stmt
        .query_map(params, |row| {
            Ok(Chapter {
                id: row.get(0)?,
                name: row.get(1)?,
                uri: row.get(2)?,
                volumeid: row.get(3)?,
                data_id: match row.get(4) {
                    Ok(id) => id,
                    Err(_) => 0,
                },
            })
        })?
        .collect();
    res
}

fn volume_query_helper<P>(db_conn: &Connection, sql: &str, params: P) -> Result<Vec<Volume>>
where
    P: Params,
{
    let mut stmt = db_conn.prepare(sql)?;
    let res = stmt
        .query_map(params, |row| {
            Ok(Volume {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect();
    res
}

pub fn get_chapter_data(db_conn: &Connection, chapter_id: usize) -> Result<String> {
    let res = db_conn.query_row(
        "SELECT data FROM raw_data WHERE chapter_id = ?1",
        [chapter_id],
        |row| row.get(0),
    )?;
    Ok(res)
}

pub fn get_chapters_by_volume(db_conn: &Connection, volume_id: usize) -> Result<Vec<Chapter>> {
    let res = chapter_query_helper(
        db_conn,
        "SELECT id, name, uri, volumeid, data_id FROM chapters WHERE volumeid = ?1",
        [volume_id],
    );
    res
}

pub fn get_empty_chapters(db_conn: &Connection) -> Result<Vec<Chapter>> {
    let res = chapter_query_helper(
        db_conn,
        "SELECT id, name, uri, volumeid, data_id FROM chapters WHERE data_id IS NULL",
        [],
    );
    res
}

pub fn get_chapters_to_regenerate(db_conn: &Connection) -> Result<Vec<Chapter>> {
    let res = chapter_query_helper(
        db_conn,
        "SELECT id, name, uri, volumeid, data_id FROM chapters WHERE regenerate_epub = 1",
        [],
    );
    res
}

pub fn get_volumes_to_regenerate(db_conn: &Connection) -> Result<Vec<Volume>> {
    let res = volume_query_helper(
        db_conn,
        "SELECT id, name FROM volumes WHERE regenerate_epub = 1",
        [],
    );
    res
}

pub fn update_generated_volume(db_conn: &Connection, id: usize, regenerate: bool) -> Result<()> {
    db_conn
        .prepare("UPDATE volumes SET regenerate_epub = ?1 WHERE id = ?2")?
        .execute([regenerate as usize, id])?;
    Ok(())
}

pub fn update_generated_chapter(db_conn: &Connection, id: usize, regenerate: bool) -> Result<()> {
    db_conn
        .prepare("UPDATE chapters SET regenerate_epub = ?1 WHERE id = ?2")?
        .execute([regenerate as usize, id])?;
    Ok(())
}
