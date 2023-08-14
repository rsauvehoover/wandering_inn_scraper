use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;
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
    toc_url: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("(Re)Building index");

    let soup = get_html(toc_url.to_string()).await?;

    for volume in soup.class("volume-wrapper").find_all() {
        let volume_title = volume.tag("h2").find().unwrap().text();
        let volume_id: usize = db::add_volume(db_conn, &volume_title)?;
        let mut count = 0;
        for chapter in volume.class("chapter-entry").find_all() {
            let a = chapter.tag("a").find().unwrap();
            let uri = a.get("href").unwrap();
            let title = a.text();
            db::add_chapter(db_conn, title, uri, volume_id)?;
            count += 1;
        }
        println!("Indexed {volume_title} with {count} chapters");
    }

    println!("Finished building index");

    Ok(())
}

async fn download_chapter(
    db_conn: &Connection,
    chapter: db::Chapter,
) -> Result<(), Box<dyn std::error::Error>> {
    let soup = get_html(chapter.uri).await?;
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
        chapter.id,
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

pub async fn download_all_chapters(
    db_conn: &Connection,
    delay: &u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let chapters = db::get_empty_chapters(db_conn)?;

    if chapters.len() == 0 {
        println!("No chapters to download");
    } else {
        println!("Downloading {} missing chapters", chapters.len());
    }
    let mut count = 0;
    for chapter in chapters {
        if count % 10 == 0 && count != 0 {
            println!("Downloaded {} chapters", count);
        }
        thread::sleep(Duration::from_millis(*delay));
        download_chapter(db_conn, chapter).await?;
        count += 1;
    }
    println!(
        "{}",
        if count > 0 {
            format!("Done downloading {count} chapters")
        } else {
            format!("No chapters to download")
        }
    );
    Ok(())
}
