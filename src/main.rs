extern crate imap;
extern crate native_tls;

// cli
use clap::Parser;

mod config;
mod imap_tools;
mod rules;
use crate::imap_tools::*;

use clap;

#[derive(Parser,Default,Debug)]
#[clap(name="IMAP sorter", author = "mfutech", version = "1.0.0", 
  about = "Process email in IMAP Inbox according to rules")]
struct Args {
    #[clap(short, long, default_value="config.ini")]
    config: String,
    #[clap(short, long, default_value="rules.yaml")]
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

    let rules = match rules::Rules::load(args.rules.as_str()) {
        Ok(rules) => rules.rules,
        Err(error) => panic!("cannot read rules : {}", error),
    };

    // connecting to IMAP server
    let domain = config.imap_server.as_str();
    let port: u16 = config.imap_port;
    let username = config.imap_username;
    let password = config.imap_password;
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
    for rule in rules {
        println!(
            "processing : {:<20}filter: {}, target: {}",
            rule.name, rule.filter, rule.target
        );
        match search_and_move(&mut imap_session, rule, args.nomove) {
            Ok(success) => println!("{}", success.unwrap()),
            Err(failed) => println!("FAILED: {:?}", failed),
        }
    }

    // be nice to the server and log out
    imap_session.logout().expect("failed to logout");
}
