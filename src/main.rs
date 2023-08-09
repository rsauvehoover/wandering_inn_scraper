use log::error;
use simple_logger;
use std::process::exit;

// mod mail;
mod config;
mod index;

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let config = config::load_config();

    // let deststuff: Vec<(String, String)> = config.destinations.into_iter().map(|dest| (dest.name, dest.email)).collect();

    let conn = match index::db_open() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Error opening database: {}", e);
            exit(1);
        }
    };

    match index::update_index(&conn, config.toc_url).await {
        Ok(_) => (),
        Err(e) => error!("Error updating index: {}", e),
    }

    match index::get_all_chapters(&conn, config.request_delay).await {
        Ok(_) => (),
        Err(e) => error!("Error getting chapters: {}", e),
    }
}
