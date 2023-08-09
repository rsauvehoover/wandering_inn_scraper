use log::error;
use simple_logger;
use std::path::Path;
use std::process::exit;

// mod mail;
mod config;
mod db;
mod epub;
mod scraper;

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let config = config::load_config();

    // let deststuff: Vec<(String, String)> = config.destinations.into_iter().map(|dest| (dest.name, dest.email)).collect();

    let conn = match db::open() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Error opening database: {}", e);
            exit(1);
        }
    };

    match scraper::update_index(&conn, config.toc_url).await {
        Ok(_) => (),
        Err(e) => error!("Error updating index: {}", e),
    }

    match scraper::download_all_chapters(&conn, config.request_delay).await {
        Ok(_) => (),
        Err(e) => error!("Error getting chapters: {}", e),
    }

    match epub::generate_epubs(&conn, Path::new("build/")) {
        Ok(_) => (),
        Err(e) => error!("Error generating epubs: {}", e),
    }
}
