use crate::rules;
use imap::ImapConnection;
use imap_proto::types::Address;
use std::borrow::Cow;

// #[derive(Default, Debug)]
// struct Enveloppe {
//     date: String,
//     subject: String,
//     from: String,
//     sender: String,
//     mailbox: String,
//     host: String,
//     reply_to: String,
//     cc: String,
//     bcc: String,
//     in_reply_to: String,
//     message_id: String,
// }

trait AddressExt {
    fn to_formated(&self) -> String;
}

impl AddressExt for imap_proto::types::Address<'_> {
    fn to_formated(&self) -> String {
        // extract mailbox and host and concatenate with a @
        format!(
            // target format
            "{}{}{}@{}",
            // get name
            match self.name.as_ref() {
                Some(buffer) => format!("<{}> ", String::from_utf8_lossy(buffer.as_ref())),
                None => "".into(),
            },
            // get mailbox
            match self.mailbox.as_ref() {
                Some(buffer) => String::from_utf8_lossy(buffer.as_ref()),
                None => "?".into(),
            },
            // get additionnal routing information
            match self.adl.as_ref() {
                Some(buffer) => String::from_utf8_lossy(buffer.as_ref()),
                None => "".into(),
            },
            // get host
            match self.host.as_ref() {
                Some(buffer) => String::from_utf8_lossy(buffer.as_ref()),
                None => "?".into(),
            },
        )
    }
}

fn get_addresses(addresses_vec_opt: Option<&Vec<Address>>) -> String {
    match addresses_vec_opt {
        // scan all Vec<Addresses<>> and make a string
        // of all addreeses in one string coma separated
        Some(addresses_vec) => {
            addresses_vec
                // goes though all addresses
                .iter()
                // properly format addreses
                .map(|addr| addr.to_formated())
                // collect resutl in a Vec<String>
                .collect::<Vec<String>>()
                // join them in a string, separated by ,
                .join(", ")
        }
        _ => "UNKNOWN".to_string(),
    }
}

pub fn search_and_move(
    imap_session: &mut imap::Session<Box<dyn ImapConnection>>,
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
        log::info!("nothing to move: {}", rule.name_and_tag());
        log::debug!("nothing to move :{}", rule.name_and_tag());
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

    if log::log_enabled!(log::Level::Debug) {
        let messages = imap_session.fetch(search.clone(), "ALL")?;
        // we are in debug mode, let's get all details of messages we are going to move properly formated

        // print header of found mails
        log::trace!(
            "{date:<22} {subject:<40} {from:<30} {to:<30}",
            date = "date",
            subject = "subject",
            from = "from",
            to = "to"
        );

        // create a decent value as default for missing header parts
        let mydefault: &Cow<'_, [u8]> = &Cow::Borrowed("-".as_bytes());
        // iterate on all message an print them
        for message in messages.iter() {
            let envelope = message
                .envelope()
                .expect("message missing envelope")
                .to_owned();

            let date =
                String::from_utf8_lossy(envelope.date.as_ref().unwrap_or(mydefault).as_ref());

            // subject more likely to not me utf8
            let subject =
                String::from_utf8_lossy(envelope.subject.as_ref().unwrap_or(mydefault).as_ref());
            let from_addresses = get_addresses(envelope.from.as_ref());
            let to_addresses = get_addresses(envelope.to.as_ref());

            log::trace!(
                "{date:<22} {subject:<40} {from:<30} {to:<30}",
                date = date.chars().take(22).collect::<String>(),
                subject = subject.chars().take(40).collect::<String>(),
                from = from_addresses.chars().take(30).collect::<String>(),
                to = to_addresses.chars().take(30).collect::<String>()
            );
        }
    };
    // do the actual move or not according to flags and set return a message
    let result = if (rule.enable && !nomove) || force {
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

    log::info!("{}", result);

    Ok(Some(result))
}
