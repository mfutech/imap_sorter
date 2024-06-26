#[allow(unused_imports)]
#[allow(dead_code)]
extern crate imap;
extern crate securestore;
// use std::collections::HashSet;
use std::path::Path;
use std::process::exit;

// cli
use clap::Parser;

#[path = "../src/config.rs"]
mod config;
#[path = "../src/imap_tools.rs"]
mod imap_tools;
#[path = "../src/rules.rs"]
mod rules;

use clap;

#[derive(Parser, Default, Debug)]
#[clap(
    name = "IMAP sorter",
    author = "mfutech",
    version = "1.0.0",
    about = "Process email in IMAP Inbox according to rules"
)]
struct Args {
    #[clap(
        short,
        long,
        default_value = "config.ini",
        help = "where to find config file"
    )]
    config: String,
    #[clap(short, long, help = "much more details about what is going on")]
    debug: bool,
    #[clap(short, long, help = "message id to fetch")]
    msgid: Option<i64>,
    filter: String,
}

fn main() {
    // let's get the argument we are called with
    let args = Args::parse();
    // setup logging according to log level (default is INFO)
    // env_logger::init();
    let logfilter = if args.debug {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    };
    env_logger::builder().filter_level(logfilter).init();

    println!("--- print all header of first message in inbox");

    let config: config::Configuration = match confy::load_path(args.config) {
        Ok(config) => config,
        Err(err) => {
            panic!("Failed to load configuration: {}", err);
        }
    };

    // connect to secret manager
    let key_file = Path::new(config.key_path.as_str());
    let secret_manager = securestore::SecretsManager::load(
        config.secure_store_path,
        securestore::KeySource::File(key_file),
    )
    .expect("Failed to load SecureStore vault!");

    // connecting to IMAP server, using parameter from vault (config.json) if exsit if not try config.ini
    let domain = match secret_manager.get("imap_server") {
        Ok(hostname) => hostname,
        Err(_) => config.imap_server,
    };
    let domain = domain.as_str();
    let port: u16 = config.imap_port;
    let username = match secret_manager.get("imap_username") {
        Ok(username) => username,
        Err(_) => config.imap_username,
    };
    let password = match secret_manager.get("imap_password") {
        Ok(password) => password,
        Err(_) => config.imap_password.clone(),
    };

    // we pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.

    let client = match imap::ClientBuilder::new(domain, port).connect() {
        Ok(client) => client,
        Err(error) => {
            log::error!("Error with IMAP server : {}", error);
            return;
        }
    };

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = client
        .login(username, password)
        .map_err(|e| e.0)
        .expect("cannot connect to IMAP server");

    // just get one message from inbox and print all details message header

    // examine inbox (read only)
    imap_session.select("INBOX").unwrap();

    let search_set = imap_session.search(args.filter).expect("search failed");
    if search_set.len() == 0 {
        println!("no message found");
        exit(-1);
    }

    let message_id = search_set.iter().next().unwrap().to_string();
    println!("message_id: {:?}", message_id);

    let messages = imap_session.fetch(message_id, "(ENVELOPE RFC822 BODY[HEADER])");
    let messages = match messages {
        Ok(messages) => messages,
        Err(error) => {
            let err = match error {
                imap::Error::Parse(parse_err) => match parse_err {
                    imap::error::ParseError::Invalid(invalid) => {
                        std::str::from_utf8(&invalid).unwrap().to_string()
                    }
                    _ => todo!(),
                },
                _ => format!("{:?}", error),
            };
            panic!("fetch return erronous result : {:?}", err);
        }
    };
    let message = if let Some(m) = messages.iter().next() {
        m
    } else {
        panic!("no message");
    };

    // let envelope = message.envelope().expect("no envelope in this message");
    // println!("-- envelope returned : {:?}", envelope);

    let header = match message.header() {
        Some(header) => std::str::from_utf8(header)
            .expect("header was not valid utf-8")
            .to_string(),
        None => "".to_string(),
    };
    println!("header: {:?}", header);
    /*
        let envelope = std::str::from_utf8(envelope)
            .expect("header was not valid utf-8")
            .to_string();

        println!("Enveloppe:\n{}", envelope);
    */
    let flags = message.flags();
    println!("flags: {:?}", flags);

    println!(
        "message : \n{}",
        String::from_utf8_lossy(message.body().expect("nobodyhome"))
    );

    // be nice to the server and log out
    imap_session.logout().expect("failed to logout");
}
