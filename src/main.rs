extern crate imap;
#[cfg(feature = "securestore")]
extern crate securestore;
#[cfg(feature = "securestore")]
use std::path::Path;
// cli
use clap::Parser;

// log
use std::io::Write;

mod config;
mod imap_tools;
mod rules;
use crate::imap_tools::*;
// use crate::rules::Rule;

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
    #[clap(short, long, help = "where to file rule YAML file")]
    rules: Option<String>,
    #[clap(short, long, help = "do not move message (aka simlation mode)")]
    nomove: bool,
    #[clap(short, long, help = "force, execute all rules, even disabled one")]
    force: bool,
    #[clap(short, long, help = "no output")]
    silent: bool,
    #[clap(short, long, help = "more details about what is going on")]
    verbose: bool,
    #[clap(short, long, help = "much more details about what is going on")]
    debug: bool,
    #[clap(
        short,
        long,
        help = "filter by this tag, only rule matching this tag will be executed"
    )]
    tag: Option<String>,
    #[clap(short = 'F', long, help = "limit to processe only designated folder")]
    folder: Option<String>,
    #[clap(short, long, help = "apply rules assigned to INBOX on provided folder")]
    onfolder: Option<String>,
    #[clap(long, help = "list all rules")]
    listrules: bool,
    #[clap(long, help = "list all tags")]
    listtags: bool,
    #[clap(long, help = "list all folders")]
    listfolders: bool,
}

fn setup_logging(args: &Args) {
    // setup logging according to log level (default is INFO)
    // env_logger::init();
    let logfilter = if args.silent {
        log::LevelFilter::Warn
    } else if args.verbose {
        // for verbose level we actully use debug logging
        log::LevelFilter::Debug
    } else if args.debug {
        // debug purpose, active the trace logging level
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    };
    env_logger::builder()
        .filter_level(logfilter)
        .format(|buf, record| {
            // we make the "info" or "debug" logging be straight output
            if record.level() == log::Level::Info || record.level() == log::Level::Debug {
                writeln!(buf, "{}", record.args())
            } else {
                // otherwise print with log level information
                writeln!(buf, "{}: {}", record.level(), record.args())
            }
        })
        .init();
}

fn main() {
    // let's get the argument we are called with
    let args = Args::parse();

    setup_logging(&args);

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

    // if only list folders, then only list folders and exit
    if args.listfolders {
        println!("folders : {}", rules_set.list_folders().join(", "));
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

    match args.folder {
        // have been provided with a folder,
        Some(folder_name) => {
            let inbox = "INBOX".to_string();
            let rules = rules_set.rules_for_folder(&inbox);
            log::info!("Applying rules for {} on folder: {}", inbox, folder_name);
            apply_rules_on_folder(
                &mut imap_session,
                rules,
                &folder_name,
                &args.tag,
                args.nomove,
                args.force,
            )
        }
        // let's do if for all folders
        None => {
            // now for each rules we find message and moved them as necessary
            for folder_name in rules_set.list_folders() {
                // test if we have a folder filter
                if let Some(folder_filter) = args.folder.as_deref() {
                    // skip this iteration if we do not match filter
                    if !folder_name.eq_ignore_ascii_case(folder_filter) {
                        log::debug!("-------------------- Skipping folder : {}", &folder_name);
                        continue;
                    }
                }
                let rules = rules_set.rules_for_folder(&folder_name);
                apply_rules_on_folder(
                    &mut imap_session,
                    rules,
                    &folder_name,
                    &args.tag,
                    args.nomove,
                    args.force,
                )
            }
        }
    }
    // be nice to the server and log out
    imap_session.logout().expect("failed to logout");
}
