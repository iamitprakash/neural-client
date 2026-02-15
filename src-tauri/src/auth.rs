use keyring::Entry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub email: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
}

pub fn save_password(email: &str, password: &str) -> Result<(), String> {
    let entry = Entry::new("antigravity-mail", email).map_err(|e| e.to_string())?;
    entry.set_password(password).map_err(|e| e.to_string())
}

pub fn get_password(email: &str) -> Result<String, String> {
    let entry = Entry::new("antigravity-mail", email).map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| e.to_string())
}

pub fn delete_password(email: &str) -> Result<(), String> {
    let entry = Entry::new("antigravity-mail", email).map_err(|e| e.to_string())?;
    entry.delete_password().map_err(|e| e.to_string())
}
