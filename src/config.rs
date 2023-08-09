use log::info;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MailConfig {
    pub name: String,
    pub address: String,
    pub password: String,
    pub smtp_hostname: String,
    pub smtp_port: u16,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserConfig {
    pub name: String,
    pub email: String,
    pub strip_colour: bool,
    pub patreon_sub: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    pub mail: MailConfig,
    pub destinations: Vec<UserConfig>,
    pub toc_url: String,
    // number of seconds to wait before allowing another request to be made
    // avoids being ip banned
    pub request_delay: u64,
}

pub fn load_config() -> Config {
    match std::fs::read_to_string("config.json") {
        Ok(str) => match serde_json::from_str::<Config>(&str) {
            Ok(config) => {
                info!("Loaded config");
                info!("Delay is {}ms", config.request_delay);
                info!(
                    "Sending from <{}> at <{}>",
                    config.mail.name, config.mail.address
                );
                for dest in &config.destinations {
                    info!("Sending to <{}> at <{}>", dest.name, dest.email);
                }
                config
            }
            Err(e) => {
                panic!("{}", e);
            }
        },
        Err(e) => {
            panic!("{}", e);
        }
    }
}
