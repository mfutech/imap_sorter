extern crate imap;
//extern crate imap_proto;
extern crate native_tls;
use std::path::Path;

mod config;

fn fetch_inbox_top(conf: config::Configuration) -> imap::error::Result<Option<String>> {
    let domain = conf.imap_server.as_str();
    let port: u16 = conf.imap_port;
    let username = conf.imap_username;
    let password = conf.imap_password;
    let tls = native_tls::TlsConnector::builder().build().unwrap();

    // we pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.
    let client = imap::connect((domain, port), domain, &tls).unwrap();

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = client.login(username, password).map_err(|e| e.0)?;

    // we want to fetch the first email in the INBOX mailbox
    imap_session.select("INBOX")?;

    // fetch message number 1 in this mailbox, along with its RFC822 field.
    // RFC 822 dictates the format of the body of e-mails
    let messages = imap_session.fetch("1:10", "ALL")?;
    for message in &messages {
        /*
        let message = if let Some(m) = messages.iter().next() {
            m
        } else {
            return Ok(None);
        };
        */
        //println!("{:?}", message);

        let envelope = message.envelope().unwrap();
        //println!("envelope: {:?}", envelope);

        let date = std::str::from_utf8(envelope.date.unwrap()).unwrap();
        println!("date: {}", date);

        let subject: &str = std::str::from_utf8(envelope.subject.unwrap()).unwrap();
        println!("subject: {}", subject);

        let froms = envelope.from.as_ref().unwrap();
        //println!("froms {:?}", froms);
        let from = std::str::from_utf8(froms[0].mailbox.unwrap()).unwrap();
        let from_host = std::str::from_utf8(froms[0].host.unwrap()).unwrap();
        println!("from {}@{}", from, from_host);

        let sender = envelope.sender.as_ref().unwrap();
        let sender_mailbox = std::str::from_utf8(sender[0].mailbox.unwrap()).unwrap();
        let sender_host = std::str::from_utf8(sender[0].host.unwrap()).unwrap();

        println!("sender {}@{}", sender_mailbox, sender_host);
    }
    //let Some(froms) = envelope.from;
    //println!("{:?}", froms);
    /*
        let from = if let Some(f) = froms.iter().next() {
            f
        } else {
            return Ok(None);
        };
        println!("from: {:?}", from);
        //println!("from: {}", from);
    */
    // extract the message's body
    /*
        let body = message.body().expect("message did not have a body!");
        let body = std::str::from_utf8(body)
            .expect("message was not valid utf-8")
            .to_string();
    */
    // be nice to the server and log out
    imap_session.logout()?;

    Ok(Some("".to_string()))
}

fn main() {
    let config_file = Path::new("config.ini");
    let config: config::Configuration = match confy::load_path(config_file) {
        Ok(config) => config,
        Err(err) => {
            panic!("Failed to load configuration: {}", err);
        }
    };
    match fetch_inbox_top(config) {
        Ok(_success) => println!("hello"),
        Err(failed) => println!("{:?}", failed),
    };
}
