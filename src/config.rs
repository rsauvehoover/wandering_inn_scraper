use log::info;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct MailConfig {
    pub name: String,
    pub address: String,
    pub password: String,
    pub smtp_hostname: String,
    pub smtp_port: u16,
    pub destinations: Vec<UserConfig>,
}
impl Default for MailConfig {
    fn default() -> Self {
        MailConfig {
            name: String::default(),
            address: String::default(),
            password: String::default(),
            smtp_hostname: "smtp.gmail.com".to_string(),
            smtp_port: 587,
            destinations: Vec::<UserConfig>::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct UserConfig {
    pub name: String,
    pub email: String,
    pub strip_colour: bool,
    pub send_full_volumes: bool,
}
impl Default for UserConfig {
    fn default() -> Self {
        UserConfig {
            name: String::default(),
            email: String::default(),
            strip_colour: false,
            send_full_volumes: false,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Config {
    pub mail: MailConfig,
    pub toc_url: String,
    // number of seconds to wait before allowing another request to be made
    // avoids being ip banned
    pub request_delay: u64,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            toc_url: "https://wanderinginn.com/table-of-contents/".to_string(),
            request_delay: 1000,
            mail: MailConfig::default(),
        }
    }
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
                for dest in &config.mail.destinations {
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
