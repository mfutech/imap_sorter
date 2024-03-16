extern crate imap;
extern crate native_tls;
extern crate securestore;
use std::path::Path;

mod "../src/config";


fn main() {
    // let's get the argument we are called with


    let config: config::Configuration = match confy::load_path(args.config) {
        Ok(config) => config,
        Err(err) => {
            panic!("Failed to load configuration: {}", err);
        }
    };
    let rules_path = match args.rules {
        Some(path) => path,
        None => config.rules_conf_path,
    };

    log::debug!("rules path: {}", rules_path);

    let rules_set = match rules::RulesSet::load(rules_path.as_str()) {
        Ok(rules_set) => rules_set,
        Err(error) => panic!("cannot read rules : {}", error),
    };

    // if only list rules, then only liste rules and exit
    if args.listrules {
        rules_set.print();
        return;
    };

    // if only list tags, then only liste tags and exit
    if args.listtags {
            println!("tags : {}", rules_set.list_tags().join(", "));
        return;
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
    let tls = native_tls::TlsConnector::builder().build().unwrap();

    // we pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.
    let client = match imap::connect((domain, port), domain, &tls) {
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


    
    
        imap_session.select("INBOX").unwrap();

    let messages = imap_session.fetch("1", "ALL").unwrap();
    for message in &messages {
        let envelope = message.envelope().expect("message missing envelope");
        print!("{:?}", envelope);

    };


    // be nice to the server and log out
    imap_session.logout().expect("failed to logout");
}
