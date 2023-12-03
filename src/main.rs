extern crate imap;
extern crate native_tls;
extern crate securestore;
use std::path::Path;

// cli
use clap::Parser;

mod config;
mod imap_tools;
mod rules;
use crate::imap_tools::*;

use clap;

#[derive(Parser, Default, Debug)]
#[clap(
    name = "IMAP sorter",
    author = "mfutech",
    version = "1.0.0",
    about = "Process email in IMAP Inbox according to rules"
)]
struct Args {
    #[clap(short, long, default_value = "config.ini")]
    config: String,
    #[clap(short, long, default_value = "rules.yaml")]
    rules: String,
    #[clap(short, long)]
    nomove: bool,
}

fn main() {
    // let's get the argument we are called with
    let args = Args::parse();

    let config: config::Configuration = match confy::load_path(args.config) {
        Ok(config) => config,
        Err(err) => {
            panic!("Failed to load configuration: {}", err);
        }
    };

    let folders_rules = match rules::RulesSet::load(args.rules.as_str()) {
        Ok(rules_set) => rules_set.folders,
        Err(error) => panic!("cannot read rules : {}", error),
    };

    // connect to secret manager
    let key_file = Path::new("secrets.key");
    let secret_manager =
        securestore::SecretsManager::load("config.json", securestore::KeySource::File(key_file))
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
    let tls = native_tls::TlsConnector::builder().build().unwrap();

    // we pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.
    let client = match imap::connect((domain, port), domain, &tls) {
        Ok(client) => client,
        Err(error) => {
            println!("Error with IMAP server : {}", error);
            return;
        }
    };

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = client
        .login(username, password)
        .map_err(|e| e.0)
        .expect("cannot connect to IMAP server");

    // now for each rules we find message and moved them as necessary
    for folder in folders_rules {
        let folder_name = folder.folder;
        println!(
            "-------------------- Processing for {} ----------",
            folder_name
        );
        for rule in folder.rules {
            println!(
                "processing : {:<20}filter: {}, target: {}",
                rule.name, rule.filter, rule.target
            );
            match search_and_move(&mut imap_session, rule, folder_name.clone(), args.nomove) {
                Ok(success) => println!("{}", success.unwrap()),
                Err(failed) => println!("FAILED: {:?}", failed),
            }
        }
    }
    // be nice to the server and log out
    imap_session.logout().expect("failed to logout");
}
