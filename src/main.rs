use log::error;
use simple_logger;

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
            panic!();
        }
    };
    match index::update_index(config.toc_url, &conn).await {
        Ok(_) => (),
        Err(e) => error!("Error updating index: {}", e),
    }
}
