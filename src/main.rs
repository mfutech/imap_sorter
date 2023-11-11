extern crate imap;
//extern crate imap_proto;
extern crate native_tls;
use std::path::Path;

mod config;
mod rules;

fn search_and_move(
    imap_session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
    rule: rules::Rule,
) -> imap::error::Result<Option<String>> {
    // we want to fetch the first email in the INBOX mailbox
    imap_session.select("INBOX")?;

    // fetch message number 1 in this mailbox, along with its RFC822 field.
    // RFC 822 dictates the format of the body of e-mails
    println!("search for : {}", rule.filter);
    let search_set = imap_session.search(rule.filter).unwrap();
    if search_set.len() == 0 {
        return Ok(Some("nothing to move".to_string()))
    }

    // println!("search set: {:?}", search_set);

    let search_vec: Vec<u32> = search_set.clone().into_iter().collect();
    let search: String = search_vec
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let messages = imap_session.fetch(search.clone(), "ALL")?;
    //println!("messages: {:?}", messages);

    for message in &messages {
        let envelope = message.envelope().expect("message missing envelope");
        //printlnln!("envelope: {:?}", envelope);

        let date = std::str::from_utf8(envelope.date.expect("envelope missing date"))
            .expect("Enveloppe date not UTF8");

        // subject more likely to not me utf8
        let subject =
            match std::str::from_utf8(envelope.subject.expect("envelopem missing subject")) {
                Ok(subject) => subject.to_string(),
                Err(error) => format!("Enveloppe subject not UTF8 : {}", error),
            };

        let froms = envelope.from.as_ref().expect("envelope missing from");
        //printlnln!("froms {:?}", froms);
        let from = std::str::from_utf8(froms[0].mailbox.unwrap()).unwrap();
        let from_host = std::str::from_utf8(froms[0].host.unwrap()).unwrap();

        let sender = envelope.sender.as_ref().unwrap();
        let sender_mailbox = std::str::from_utf8(sender[0].mailbox.unwrap()).unwrap();
        let sender_host = std::str::from_utf8(sender[0].host.unwrap()).unwrap();

        println!(
            "date: {}\t subject: {}\t from:{}@{}\t sender: {}@{}",
            date, subject, from, from_host, sender_mailbox, sender_host
        );
    }

    // extract the message's body
    /*
        let body = message.body().expect("message did not have a body!");
        let body = std::str::from_utf8(body)
            .expect("message was not valid utf-8")
            .to_string();
    */
    if rule.enable {
        imap_session.mv(search, rule.target)?;
    } else {
        println!("rule disabled, skipping move")
    }

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
    let domain = config.imap_server.as_str();
    let port: u16 = config.imap_port;
    let username = config.imap_username;
    let password = config.imap_password;
    let tls = native_tls::TlsConnector::builder().build().unwrap();

    // we pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.
    let client = imap::connect((domain, port), domain, &tls).unwrap();

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = client
        .login(username, password)
        .map_err(|e| e.0)
        .expect("cannot connect to IMAP server");

    let rules = match rules::Rules::load("rules.yaml") {
        Ok(rules) => rules.rules,
        Err(error) => panic!("cannot read rules : {}", error),
    };

    for rule in rules {
        println!("processing : {}", rule.name);
        println!("filter : {}", rule.filter);
        println!("target : {}", rule.target);
        match search_and_move(&mut imap_session, rule) {
            Ok(success) => println!("{:?}", success),
            Err(failed) => println!("FAILED: {:?}", failed),
        }
    }

    // be nice to the server and log out
    imap_session.logout().expect("failed to logout");
}
