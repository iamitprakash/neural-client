use rusqlite::{params, Connection, Result}; // Assuming Email struct is exported/usable here, or we redefine, but Slint generates Email.

// We will redefine a basic struct to avoid fighting with Slint's generated Rc/Model types in DB threads
#[derive(Debug, Clone)]
pub struct DbEmail {
    pub id: i32,
    pub subject: String,
    pub sender: String,
    pub date: String,
    pub body: String,
    pub has_attachment: bool,
}

pub fn init_db() -> Result<()> {
    let conn = Connection::open("neural-mail.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS emails (
            id INTEGER PRIMARY KEY,
            subject TEXT NOT NULL,
            sender TEXT NOT NULL,
            date_str TEXT NOT NULL,
            body TEXT NOT NULL,
            has_attachment INTEGER NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub fn insert_emails(emails: &[DbEmail]) -> Result<()> {
    let mut conn = Connection::open("neural-mail.db")?;
    let tx = conn.transaction()?;

    // Clear existing to avoid duplicates on every run
    tx.execute("DELETE FROM emails", [])?;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO emails (id, subject, sender, date_str, body, has_attachment) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;

        for email in emails {
            stmt.execute(params![
                email.id,
                email.subject,
                email.sender,
                email.date,
                email.body,
                if email.has_attachment { 1 } else { 0 }
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn get_all_emails() -> Result<Vec<DbEmail>> {
    let conn = Connection::open("neural-mail.db")?;
    let mut stmt =
        conn.prepare("SELECT id, subject, sender, date_str, body, has_attachment FROM emails")?;
    let email_iter = stmt.query_map([], |row| {
        Ok(DbEmail {
            id: row.get(0)?,
            subject: row.get(1)?,
            sender: row.get(2)?,
            date: row.get(3)?,
            body: row.get(4)?,
            has_attachment: row.get::<_, i32>(5)? == 1,
        })
    })?;

    let mut emails = Vec::new();
    for email in email_iter {
        emails.push(email?);
    }
    Ok(emails)
}

pub fn count_emails() -> Result<i64> {
    let conn = Connection::open("neural-mail.db")?;
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM emails")?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        let count: i64 = row.get(0)?;
        Ok(count)
    } else {
        Ok(0)
    }
}

pub fn save_sidebar_width(width: f32) -> Result<()> {
    let conn = Connection::open("neural-mail.db")?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('sidebar_width', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![width.to_string()],
    )?;
    Ok(())
}

pub fn get_sidebar_width() -> Result<f32> {
    let conn = Connection::open("neural-mail.db")?;
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'sidebar_width'")?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        let val_str: String = row.get(0)?;
        if let Ok(width) = val_str.parse::<f32>() {
            return Ok(width);
        }
    }
    // Default width
    Ok(570.0)
}

pub fn save_theme_mode(mode: &str) -> Result<()> {
    let conn = Connection::open("neural-mail.db")?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('theme_mode', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![mode],
    )?;
    Ok(())
}

pub fn get_theme_mode() -> Result<String> {
    let conn = Connection::open("neural-mail.db")?;
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'theme_mode'")?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        let val_str: String = row.get(0)?;
        return Ok(val_str);
    }
    // Default mode
    Ok("system".to_string())
}
