use crate::rules;
use imap_proto::types::Address;

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

pub fn search_and_move(
    imap_session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
    rule: rules::Rule,
    folder: String,
    nomove: bool,
    force: bool,
) -> imap::error::Result<Option<String>> {
    // we want to fetch the first email in the INBOX mailbox
    imap_session.select(folder)?;

    // fetch message number 1 in this mailbox, along with its RFC822 field.
    // RFC 822 dictates the format of the body of e-mails
    let search_set = imap_session.search(rule.filter.clone())?;
    if search_set.len() == 0 {
        return Ok(Some("nothing to move".to_string()));
    }

    log::info!("processing :{}", rule.name_and_tag());
    log::debug!("{}", rule.as_string());

    // collect found message to create a reference string for fetch
    let search_vec: Vec<u32> = search_set.clone().into_iter().collect();
    let search: String = search_vec
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let messages = imap_session.fetch(search.clone(), "ALL")?;

    if log::log_enabled!(log::Level::Debug) {
        // we are in debug mode, let's get all details of messages we are going to move properly formated

        // print header of found mails
        log::debug!(
            "{date:<22} {subject:<40} {from:<30} {to:<30}",
            date = "date",
            subject = "subject",
            from = "from",
            to = "to"
        );

        // iterate on all message an print them
        for message in &messages {
            let envelope = message.envelope().expect("message missing envelope");

            let date = match envelope.date {
                Some(date) => std::str::from_utf8(date).expect("Enveloppe date not UTF8"),
                None => "NODATE",
            };

            // subject more likely to not me utf8
            let subject =
                match std::str::from_utf8(envelope.subject.expect("envelopem missing subject")) {
                    Ok(subject) => subject.to_string(),
                    Err(error) => format!("Enveloppe subject not UTF8 : {}", error),
                };

            let from_addresses = match envelope.from.as_ref() {
                Some(froms) => get_addresses(froms).unwrap(),
                _ => "FROM_UKN".to_string(),
            };

            let to_addresses = match envelope.to.as_ref() {
                Some(tos) => get_addresses(tos).unwrap(),
                _ => "TO_UNKN".to_string(),
            };

            log::debug!(
                "{date:<22} {subject:<40} {from:<30} {to:<30}",
                date = date.chars().take(22).collect::<String>(),
                subject = subject.chars().take(40).collect::<String>(),
                from = from_addresses.chars().take(30).collect::<String>(),
                to = to_addresses.chars().take(30).collect::<String>()
            );
        }
    };
    // do the actual move or not according to flags and set return a message
    let message = if (rule.enable && !nomove) || force {
        // let's move them
        imap_session.mv(search, rule.target)?;
        // and tell them how much we worked
        format!("processed {} messages", search_set.len())
    } else {
        // skip move as rule is disabled or running in simulation
        format!(
            "rule disabled, did not process {} messages",
            search_set.len()
        )
    };

    log::info!("{}", message);

    Ok(Some(message))
}
