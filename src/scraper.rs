use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;
use log::info;
use regex::Regex;
use rusqlite::{Connection, Result};
use soup::prelude::*;

use crate::db;

use std::{thread, time::Duration};

async fn get_html(uri: String) -> Result<Soup, Box<dyn std::error::Error>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let res = client.get(uri.parse::<Uri>().unwrap()).await?;
    let bytes = hyper::body::to_bytes(res).await?;
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    Ok(Soup::new(&body))
}

pub async fn update_index(
    db_conn: &Connection,
    toc_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("(Re)Building index");

    let soup = get_html(toc_url).await?;

    for volume in soup.class("volume-wrapper").find_all() {
        let volume_title = volume.tag("h2").find().unwrap().text();
        let volume_id: usize = db::db_add_volume(db_conn, &volume_title)?;
        let mut count = 0;
        for chapter in volume.class("chapter-entry").find_all() {
            let a = chapter.tag("a").find().unwrap();
            let uri = a.get("href").unwrap();
            let title = a.text();
            db::db_add_chapter(db_conn, title, uri, volume_id)?;
            count += 1;
        }
        info!("Indexed {volume_title} with {count} chapters");
    }

    info!("Finished building index");

    Ok(())
}

async fn get_chapter(
    db_conn: &Connection,
    chapter_id: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let uri = db_conn.query_row(
        "SELECT uri FROM chapters WHERE id = ?1",
        [chapter_id],
        |row| row.get(0),
    )?;

    let soup = get_html(uri).await?;
    let html = soup.class("entry-content").find().unwrap();

    let header = "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.1//EN\" \"http://www.w3.org/TR/xhtml11    /DTD/xhtml11.dtd\">
<html xmlns=\"http://www.w3.org/1999/xhtml\">
<head>
<meta http-equiv=\"Content-Type\" content=\"text/html; charset=UTF-8\" />
<meta name=\"author\" content=\"pirate aba\"/>
<meta name=\"description\" content=\"The Wandering Inn\"/>
<meta name=\"classification\" content=\"Fantasy\" />
<title>The Wandering Inn</title>
<link rel=\"stylesheet\" href=\"style.css\" type = \"text/css\" />
</head>
<body>";

    let title = format!(
        "<h1>{}</h1>",
        soup.class("entry-title").find().unwrap().text()
    );
    let re = Regex::new(r"<a.*?</a>").unwrap();
    let body = html.display();
    let footer = "</body>";

    db::add_chapter_data(
        db_conn,
        chapter_id,
        &format!(
            "{}\n{}\n{}\n{}\n",
            header,
            title,
            re.replace_all(&body, ""),
            footer
        ),
    )?;
    Ok(())
}

pub async fn get_all_chapters(
    db_conn: &Connection,
    delay: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stmt = db_conn.prepare("SELECT id FROM chapters WHERE data_id IS NULL")?;
    let chapters = stmt.query_map([], |row| row.get::<usize, usize>(0))?;

    info!("Downloading all missing chapters");
    let mut count = 0;
    for chapter in chapters {
        if count % 10 == 0 && count != 0 {
            info!("Downloaded {} chapters", count);
        }
        thread::sleep(Duration::from_millis(delay));
        get_chapter(db_conn, chapter?).await?;
        count += 1;
    }
    info!(
        "{}",
        if count > 0 {
            format!("Done downloading {count} chapters")
        } else {
            format!("No chapters to download")
        }
    );
    Ok(())
}