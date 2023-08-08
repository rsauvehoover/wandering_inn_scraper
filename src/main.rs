// mod config;
// mod mail;

use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;

use soup::prelude::*;

#[tokio::main]
async fn main() {
    // let config = config::load_config();
    // let deststuff: Vec<(String, String)> = config.destinations.into_iter().map(|dest| (dest.name, dest.email)).collect();

    let _ = tmp().await;

    /* 
    match std::fs::read("./book.epub") {
        Ok(bytes) => {
            mail::send_book(bytes).await;
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                eprintln!("please run again with appropriate permissions.");
                return;
            }
            panic!("{}", e);
        }
    }
    */
}

async fn tmp() -> Result<(), Box<dyn std::error::Error>> {
    let toc_url = "https://wanderinginn.com/table-of-contents/";

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let res = client.get(Uri::from_static(toc_url)).await?;
    println!("Result {}", res.status());
    let bytes = hyper::body::to_bytes(res).await?;
    let body= String::from_utf8(bytes.to_vec()).unwrap();

    let soup = Soup::new(&body);
    let mut uris: Vec<Vec<String>> = Vec::new();

    // Grab all titles for volumes
    for title in soup.class("volume-wrapper").find_all() {
        println!("{:?}", title.get("id"));
        let mut local_uris: Vec<String> = Vec::new();
        // grab all chapter URIs by volume
        // TODO filter by chapter-entry class instead to avoid unecessary URIs
        for chapter in title.tag("a").find_all() {
            local_uris.push(chapter.get("href").unwrap());
        }
        uris.push(local_uris);

        // TODO: Build index formatted by volume/chapter etc. new flag means to add to next book send operation
        // i.e.
        /*
            {
                "vol-1": {
                    "1.0.0": { "uri": "https://wanderinginn.com/2017/03/03/rw1-00/", new: true},
                    ...
                }
            }
         */
    }

    println!("{:?}", uris);

    Ok(())
}