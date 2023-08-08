use mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;

pub async fn send_book(bytes: Vec<u8>) {
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