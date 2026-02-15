mod auth;
mod mail;
mod ai;

slint::include_modules!();

use slint::{ModelRc, VecModel};
use std::rc::Rc;
use tokio::runtime::Runtime;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    
    // Create a tokio runtime for async operations
    let rt = Runtime::new().unwrap();
    
    let ui_handle = ui.as_weak();
    ui.on_fetch_emails(move || {
        let ui = ui_handle.unwrap();
        
        // Mock data for demo since real IMAP setup requires credentials
        let mut emails_vec = Vec::new();
        emails_vec.push(Email {
            id: 1,
            subject: "Pure Rust is Awesome".into(),
            sender: "rust@community.org".into(),
            date: "Today".into(),
        });
        emails_vec.push(Email {
            id: 2,
            subject: "Ollama Local AI Test".into(),
            sender: "ai@local.home".into(),
            date: "Yesterday".into(),
        });
        emails_vec.push(Email {
            id: 3,
            subject: "Antigravity Native Demo".into(),
            sender: "antigravity@mail.com".into(),
            date: "Feb 16".into(),
        });
        
        let model = Rc::new(VecModel::from(emails_vec));
        ui.set_emails(ModelRc::from(model));
    });

    let ui_handle_ai = ui.as_weak();
    ui.on_summarize_email(move |content| {
        let ui = ui_handle_ai.unwrap();
        ui.set_loading(true);
        
        let content_clone = content.to_string();
        let ui_for_async = ui_handle_ai.clone();
        
        // Run AI request in the background
        rt.spawn(async move {
            let result = ai::generate_summary(&content_clone).await;
            
            slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_for_async.upgrade() {
                    ui.set_loading(false);
                    match result {
                        Ok(summary) => ui.set_ai_summary(summary.into()),
                        Err(e) => ui.set_ai_summary(format!("Ollama Error: {}. Ensure Ollama is running.", e).into()),
                    }
                }
            }).unwrap();
        });
    });

    ui.run()
}
