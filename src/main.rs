extern crate imap;
extern crate native_tls;
//use std::{path::Path, fmt::format};
use std::path::Path;
// use imap_proto;
use imap_proto::types::Address;

mod config;
mod rules;

fn get_addresses(addresses_vec: &Vec<Address<'_>>) -> Result<String, String> {
    // scan all Vec<Addresses<>> and make a string
    // of all addreeses in one string coma separated
    Ok(addresses_vec
        // goes though all addresses
        .iter()
        .map(|addr| {
            // extract mailbox and host and concatenate with a @
            format!(
                // target format
                "{}@{}",
                // get mailbox
                std::str::from_utf8(match addr.mailbox.as_ref() {
                    // if no host, replace by uknown
                    Some(mailbox) => mailbox,
                    _ => "unknown".as_bytes(),
                })
                .unwrap(),
                // get host
                std::str::from_utf8(match addr.host.as_ref() {
                    // if no host, replace by uknown
                    Some(host) => host,
                    _ => "unknown".as_bytes(),
                })
                .unwrap(),
            )
        })
        // collect resutl in a Vec<String>
        .collect::<Vec<String>>()
        // join them in a string, separated by ,
        .join(", "))
}

fn search_and_move(
    imap_session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
    rule: rules::Rule,
) -> imap::error::Result<Option<String>> {
    // we want to fetch the first email in the INBOX mailbox
    imap_session.select("INBOX")?;

    // fetch message number 1 in this mailbox, along with its RFC822 field.
    // RFC 822 dictates the format of the body of e-mails
    // println!("search for : {}", rule.filter);
    let search_set = imap_session.search(rule.filter)?;
    if search_set.len() == 0 {
        return Ok(Some("nothing to move".to_string()));
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

    // print header of found mails
    println!(
        "{date:<22} {subject:<40} {from:<30} {to:<30}",
        date = "date",
        subject = "subject",
        from = "from",
        to = "to"
    );

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

        let from_addresses = match envelope.from.as_ref() {
            Some(froms) => get_addresses(froms).unwrap(),
            _ => "FROM_UKN".to_string()
        };
            
        // let sender_addresses = get_addresses(envelope.sender.as_ref().expect("no sender in enveloppe")).unwrap();
        let to_addresses =match envelope.to.as_ref() {
            Some(tos) => get_addresses(tos).unwrap(),
            _ => "TO_UNKN".to_string()
        };

        println!(
            "{}",
            format!(
                "{date:<22} {subject:<40} {from:<30} {to:<30}",
                date = date.chars().take(22).collect::<String>(),
                subject = subject.chars().take(40).collect::<String>(),
                from = from_addresses.chars().take(30).collect::<String>(),
                to = to_addresses.chars().take(30).collect::<String>()
            )
        );
    }

    if rule.enable {
        imap_session.mv(search, rule.target)?;
    };

    let message = if rule.enable {
        format!("processed {} messages", search_set.len())
    } else {
        format!("rule disabled, did not process {} messages", search_set.len())
    };
    Ok(Some(message))
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
        println!(
            "processing : {:<20}filter: {}, target: {}",
            rule.name, rule.filter, rule.target
        );
        match search_and_move(&mut imap_session, rule) {
            Ok(success) => println!("{}", success.unwrap()),
            Err(failed) => println!("FAILED: {:?}", failed),
        }
    }

    // be nice to the server and log out
    imap_session.logout().expect("failed to logout");
}
