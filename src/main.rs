use std::path::Path;

// mod mail;
mod config;
mod db;
mod epub;
mod scraper;

#[tokio::main]
async fn main() {
    let config = config::load_config();

    // let deststuff: Vec<(String, String)> = config.destinations.into_iter().map(|dest| (dest.name, dest.email)).collect();

    let conn = match db::open() {
        Ok(conn) => conn,
        Err(e) => {
            panic!("Error opening database: {}", e);
        }
    };

    match scraper::update_index(&conn, &config.toc_url).await {
        Ok(_) => (),
        Err(e) => panic!("Error updating index: {}", e),
    }

    match scraper::download_all_chapters(&conn, &config.request_delay).await {
        Ok(_) => (),
        Err(e) => panic!("Error getting chapters: {}", e),
    }

    match epub::generate_epubs(&conn, Path::new("build/"), &config) {
        Ok(_) => (),
        Err(e) => panic!("Error generating epubs: {}", e),
    }
}
