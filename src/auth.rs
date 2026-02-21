use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result as SqlResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub email: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub password: Option<String>,
    pub is_demo: bool,
}

pub fn init_db() -> SqlResult<()> {
    let conn = Connection::open("neural-mail.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            email TEXT PRIMARY KEY,
            imap_host TEXT NOT NULL,
            imap_port INTEGER NOT NULL,
            password TEXT,
            is_demo INTEGER DEFAULT 0
        )",
        [],
    )?;
    Ok(())
}

pub fn save_account(account: &Account) -> SqlResult<()> {
    let conn = Connection::open("neural-mail.db")?;
    conn.execute(
        "INSERT OR REPLACE INTO accounts (email, imap_host, imap_port, password, is_demo) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![account.email, account.imap_host, account.imap_port, account.password, if account.is_demo { 1 } else { 0 }],
    )?;
    Ok(())
}

pub fn get_accounts() -> SqlResult<Vec<Account>> {
    let conn = Connection::open("neural-mail.db")?;
    let mut stmt = conn.prepare("SELECT email, imap_host, imap_port, password, is_demo FROM accounts")?;
    let account_iter = stmt.query_map([], |row| {
        Ok(Account {
            email: row.get(0)?,
            imap_host: row.get(1)?,
            imap_port: row.get(2)?,
            password: row.get(3)?,
            is_demo: row.get::<_, i32>(4)? == 1,
        })
    })?;

    let mut accounts = Vec::new();
    for account in account_iter {
        accounts.push(account?);
    }
    Ok(accounts)
}

pub fn save_password(email: &str, password: &str) -> Result<(), String> {
    let conn = Connection::open("neural-mail.db").map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE accounts SET password = ?1 WHERE email = ?2",
        params![password, email],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_password(email: &str) -> Result<String, String> {
    let conn = Connection::open("neural-mail.db").map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT password FROM accounts WHERE email = ?1").map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![email]).map_err(|e| e.to_string())?;
    
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let password: Option<String> = row.get(0).map_err(|e| e.to_string())?;
        password.ok_or_else(|| "No password found".to_string())
    } else {
        Err("Account not found".to_string())
    }
}
