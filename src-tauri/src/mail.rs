use imap;
use native_tls::TlsConnector;
use serde::Serialize;
use crate::auth::get_password;

#[derive(Debug, Serialize)]
pub struct EmailHeader {
    pub id: u32,
    pub subject: String,
    pub from: String,
    pub date: String,
}

pub fn fetch_inbox(
    email: &str,
    imap_host: &str,
    imap_port: u16,
) -> Result<Vec<EmailHeader>, String> {
    let password = get_password(email)?;
    
    let tls = TlsConnector::builder().build().map_err(|e| e.to_string())?;
    let client = imap::connect((imap_host, imap_port), imap_host, &tls)
        .map_err(|e| e.to_string())?;

    let mut session = client.login(email, &password).map_err(|e| e.to_string())?;
    session.select("INBOX").map_err(|e| e.to_string())?;

    let messages = session.fetch("1:10", "RFC822.SIZE ENVELOPE").map_err(|e| e.to_string())?;
    
    let mut headers = Vec::new();
    for m in &messages {
        if let Some(envelope) = m.envelope() {
            headers.push(EmailHeader {
                id: m.message,
                subject: String::from_utf8_lossy(envelope.subject.as_ref().unwrap_or(&vec![])).into_owned(),
                from: envelope.from.as_ref()
                    .and_then(|f| f.first())
                    .map(|addr| {
                        let mailbox = String::from_utf8_lossy(addr.mailbox.as_ref().unwrap_or(&vec![])).into_owned();
                        let host = String::from_utf8_lossy(addr.host.as_ref().unwrap_or(&vec![])).into_owned();
                        format!("{}@{}", mailbox, host)
                    })
                    .unwrap_or_default(),
                date: envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).into_owned()).unwrap_or_default(),
            });
        }
    }

    session.logout().map_err(|e| e.to_string())?;
    Ok(headers)
}
