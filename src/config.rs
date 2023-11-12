use serde::{Deserialize, Serialize};

// Structure holding configuration of the application
// it is linked to configuration file and is updated by the application when using -s/--save option
// managed by confy
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Configuration {
    pub imap_server: String,   // database hostname
    pub imap_port: u16,        // database port
    pub imap_username: String, // database username
    pub imap_password: String, // database password
}

impl ::std::default::Default for Configuration {
    fn default() -> Configuration {
        Configuration {
            imap_server: String::from("localhost"),
            imap_port: 993,
            imap_username: String::from("user"),
            imap_password: String::from(""),
        }
    }
}
