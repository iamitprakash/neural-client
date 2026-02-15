mod auth;
mod mail;
mod ai;

use auth::{Account, save_password};
use mail::EmailHeader;

#[tauri::command]
fn add_account(account: Account, password: String) -> Result<(), String> {
    save_password(&account.email, &password)?;
    // In a real app, we'd save the account metadata to SQLite here.
    Ok(())
}

#[tauri::command]
fn get_emails(email: String, imap_host: String, imap_port: u16) -> Result<Vec<EmailHeader>, String> {
    mail::fetch_inbox(&email, &imap_host, imap_port)
}

#[tauri::command]
async fn summarize_content(text: String) -> Result<String, String> {
    ai::generate_summary(&text).await
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            add_account,
            get_emails,
            summarize_content
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
