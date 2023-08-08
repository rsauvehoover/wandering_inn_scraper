use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;
use log::info;
use rusqlite::{Connection, Result};
use soup::prelude::*;

pub fn db_open() -> Result<Connection> {
    let conn = Connection::open("index.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS volumes(
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        UNIQUE(name)
    )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS chapters(
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        uri TEXT NOT NULL,
        processed INTEGER CHECK(processed >= 0 and processed <=1) DEFAULT 0,
        volumeid INTEGER,
        FOREIGN KEY(volumeid) REFERENCES volume(id),
        UNIQUE(name, uri, volumeid)
    )",
        (),
    )?;

    Ok(conn)
}

fn db_add_volume(db_conn: &Connection, name: &String) -> Result<usize> {
    db_conn
        .prepare("INSERT OR IGNORE INTO volumes(name) values(?1)")?
        .execute([name])?;
    Ok(
        db_conn.query_row("SELECT id FROM volumes WHERE name = ?1", [name], |row| {
            row.get(0)
        })?,
    )
}

fn db_add_chapter(db_conn: &Connection, name: String, uri: String, volume: usize) -> Result<()> {
    db_conn
        .prepare("INSERT OR IGNORE INTO chapters(name, uri, volumeid) values(?1, ?2, ?3)")?
        .execute((name, uri, volume))?;
    Ok(())
}

pub async fn update_index(
    toc_url: String,
    db_conn: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("(Re)Building index");

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let res = client.get(toc_url.parse::<Uri>().unwrap()).await?;
    let bytes = hyper::body::to_bytes(res).await?;
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    let soup = Soup::new(&body);

    for volume in soup.class("volume-wrapper").find_all() {
        let volume_title = volume.tag("h2").find().unwrap().text();
        let volume_id: usize = db_add_volume(db_conn, &volume_title)?;
        let mut count = 0;
        for chapter in volume.class("chapter-entry").find_all() {
            let a = chapter.tag("a").find().unwrap();
            let uri = a.get("href").unwrap();
            let title = a.text();
            db_add_chapter(db_conn, title, uri, volume_id)?;
            count += 1;
        }
        info!("Indexed {volume_title} with {count} chapters");
    }

    info!("Finished building index");

    Ok(())
}
