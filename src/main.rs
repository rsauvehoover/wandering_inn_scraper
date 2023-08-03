use mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MailConfig {
    name: String,
    address: String,
    password: String,
    smtp_hostname: String,
    smtp_port: u16,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UserConfig {
    name: String,
    email: String,
    strip_colour: bool,
    patreon_sub: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Config {
    mail: MailConfig,
    destinations: Vec<UserConfig>,
}

#[tokio::main]
async fn main() {
    let config = load_config();
    let deststuff: Vec<(String, String)> = config.destinations.into_iter().map(|dest| (dest.name, dest.email)).collect();

    let message = MessageBuilder::new()
        .from((
            config.mail.name,
            config.mail.address,
        ))
        .to(deststuff)
        .text_body("Hello");
        // .attachment("application/epub+zip", "image.epub", bytes);

    println!("{:?}", message);

    /*
    match std::fs::read("./book.epub") {
        Ok(bytes) => {
            send_book(bytes).await;
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

async fn send_book(bytes: Vec<u8>) {
    // Build a simple multipart message

    let message = MessageBuilder::new()
        .from((
            "Frederic Epub Delivery Service",
            "FredericEpubService@gmail.com",
        ))
        .to(vec![("Frederic Sauve-Hoover", "sauvehoover@gmail.com")])
        .attachment("application/epub+zip", "image.epub", bytes);

    let res = SmtpClientBuilder::new("smtp.gmail.com", 587)
        .implicit_tls(false)
        .credentials(("FredericEpubService@gmail.com", "this is not a real password"))
        .connect()
        .await
        .unwrap()
        .send(message)
        .await;

    match res {
        Ok(()) => println!("Success"),
        Err(error) => panic!("Problem with sending email {:?}", error),
    };
}

fn load_config() -> Config {
    match std::fs::read_to_string("config.json") {
        Ok(str) => match serde_json::from_str(&str) {
            Ok(config) => config,
            Err(e) => {
                panic!("{}", e);
            }
        },
        Err(e) => {
            panic!("{}", e);
        }
    }
}
