mod auth;
mod mail;
mod ai;
mod db;

slint::include_modules!();

use slint::{Model, ModelRc, VecModel};
use std::rc::Rc;
use tokio::runtime::Runtime;

use rand::Rng;
use rand::RngExt; // For random_range, random_bool in 0.10.0

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let rt = Runtime::new().unwrap();
    let rt_handle_ai = rt.handle().clone();
    let rt_handle_chat = rt.handle().clone();
    
    // Initialize both DBs (accounts if we ever use them, and emails)
    let _ = auth::init_db();
    let _ = db::init_db();
    
    // Generate mock emails ONLY if database is empty
    match db::count_emails() {
        Ok(count) if count == 0 => {
            println!("Database empty. Generating 1000 mock emails...");
            let mut rng = rand::rng();
            let mut db_emails = Vec::with_capacity(1000);

            let subjects = ["Project Update", "Invoice #", "Weekly Newsletter", "Meeting Notes", "Q3 Report", "Lunch?", "Action Required"];
            let senders = ["boss@company.com", "billing@services.io", "newsletter@techly.com", "team@company.com", "friend@email.com"];
            let bodies = [
                "Please find the latest project update. We are on track to deliver all features in Q4. This includes the new AI engine and the revamped UI. Let me know if you have any questions.\n\nThe team worked hard on this, so please share your feedback soon.",
                "Your invoice is due next week. Please review the attached PDF for the breakdown of the charges. If there are any discrepancies, contact billing support immediately.",
                "Here is the weekly round-up of tech news! Lots of exciting developments in AI this week. We have seen new models released and better performance benchmarks.\n\nRead the full details below.",
                "Notes from our sync this morning. Key takeaways: we need more velocity on the frontend. Backend APIs look solid but we are lacking test coverage. Everyone, please update your JIRA tickets.",
                "Just checking if you wanted to grab lunch today? I was thinking about that new place downtown.",
                "Please review the attached document and provide your sign-off by EOD. This is critical for unblocking the release."
            ];

            for i in 1..=1000 {
                let has_attachment = rng.random_bool(0.2);
                let subject_idx = rng.random_range(0..subjects.len());
                let sender_idx = rng.random_range(0..senders.len());
                let body_idx = rng.random_range(0..bodies.len());
                
                let mut subject = subjects[subject_idx].to_string();
                if subject == "Invoice #" {
                    subject = format!("Invoice #{}", rng.random_range(1000..9999));
                }
                
                let mins_ago = rng.random_range(1..60000);
                let date = if mins_ago < 60 {
                    format!("{}m ago", mins_ago)
                } else if mins_ago < 1440 {
                    format!("{}h ago", mins_ago / 60)
                } else {
                    format!("{}d ago", mins_ago / 1440)
                };

                db_emails.push(db::DbEmail {
                    id: i,
                    subject,
                    sender: senders[sender_idx].into(),
                    date,
                    body: bodies[body_idx].into(),
                    has_attachment,
                });
            }
            
            if let Err(e) = db::insert_emails(&db_emails) {
                eprintln!("Failed to insert into SQLite: {}", e);
            }
        },
        Ok(count) => println!("Found {} emails in SQLite, skipping generation.", count),
        Err(e) => eprintln!("Error checking database: {}", e),
    }
    
    let ui_handle = ui.as_weak();
    ui.on_fetch_emails(move || {
        let ui = ui_handle.unwrap();
        
        let mut slint_emails = Vec::new();
        match db::get_all_emails() {
            Ok(db_emails) => {
                for e in db_emails {
                    slint_emails.push(Email {
                        id: e.id,
                        subject: e.subject.into(),
                        sender: e.sender.into(),
                        date: e.date.into(),
                        body: e.body.into(),
                        has_attachment: e.has_attachment,
                    });
                }
                let model = Rc::new(VecModel::from(slint_emails));
                ui.set_emails(ModelRc::from(model));
                ui.set_status_message("Loaded Emails from DB".into());
            },
            Err(e) => {
                ui.set_status_message(format!("DB error: {}", e).into());
            }
        }
    });
    
    // Trigger initial fetch
    ui.invoke_fetch_emails();

    // -- Contextual Single Email Chat Backend Handle --
    let ui_handle_email_chat = ui.as_weak();
    ui.on_send_email_chat_message(move |msg| {
        let ui = ui_handle_email_chat.unwrap();
        ui.set_loading(true);
        
        let msg_clone = msg.to_string();
        let ui_for_async = ui_handle_email_chat.clone();
        
        let subject = ui.get_active_email_subject().to_string();
        let sender = ui.get_active_email_sender().to_string();
        let body = ui.get_active_email_body().to_string();

        let mut history: Vec<ChatMessage> = ui.get_email_chat_history().iter().collect();
        history.push(ChatMessage {
            is_user: true,
            text: msg_clone.clone().into(),
        });
        ui.set_email_chat_history(ModelRc::from(Rc::new(VecModel::from(history))));
        ui.set_email_chat_input("".into());

        rt_handle_ai.spawn(async move {
            let context_str = format!("From: {}\nSubject: {}\nBody: {}\n", sender, subject, body);
            
            let result = ai::chat_with_emails(&msg_clone, &context_str).await;
            
            slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_for_async.upgrade() {
                    ui.set_loading(false);
                    let mut history: Vec<ChatMessage> = ui.get_email_chat_history().iter().collect();
                    match result {
                        Ok(reply) => history.push(ChatMessage {
                            is_user: false,
                            text: reply.into(),
                        }),
                        Err(e) => history.push(ChatMessage {
                            is_user: false,
                            text: format!("Error: {}", e).into(),
                        }),
                    }
                    ui.set_email_chat_history(ModelRc::from(Rc::new(VecModel::from(history))));
                }
            }).unwrap();
        });
    });

    let ui_handle_chat = ui.as_weak();
    ui.on_send_chat_message(move |msg| {
        let ui = ui_handle_chat.unwrap();
        ui.set_loading(true);
        
        let msg_clone = msg.to_string();
        let ui_for_async = ui_handle_chat.clone();
        
        // Push user's message immediately
        let mut history: Vec<ChatMessage> = ui.get_chat_history().iter().collect();
        history.push(ChatMessage {
            is_user: true,
            text: msg_clone.clone().into(),
        });
        ui.set_chat_history(ModelRc::from(Rc::new(VecModel::from(history))));
        ui.set_chat_input("".into());

        rt_handle_chat.spawn(async move {
            let mut context_str = String::new();
            if let Ok(emails) = db::get_all_emails() {
                for e in emails.iter().take(100) { // Limit to 100 to avoid breaking limits if 1000 is still too large
                    context_str.push_str(&format!("From: {}, Subject: {}\nBody: {}\n\n", e.sender, e.subject, e.body));
                }
            } else {
                context_str = "No emails found in SQLite database.".to_string();
            }

            let result = ai::chat_with_emails(&msg_clone, &context_str).await;
            
            slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_for_async.upgrade() {
                    ui.set_loading(false);
                    let mut history: Vec<ChatMessage> = ui.get_chat_history().iter().collect();
                    match result {
                        Ok(reply) => history.push(ChatMessage {
                            is_user: false,
                            text: reply.into(),
                        }),
                        Err(e) => history.push(ChatMessage {
                            is_user: false,
                            text: format!("Error: {}", e).into(),
                        }),
                    }
                    ui.set_chat_history(ModelRc::from(Rc::new(VecModel::from(history))));
                }
            }).unwrap();
        });
    });

    let ui_handle_sidebar = ui.as_weak();
    ui.on_save_sidebar_width(move |width| {
        if let Err(e) = db::save_sidebar_width(width) {
            eprintln!("Failed to save sidebar width: {}", e);
        }
    });

    if let Ok(width) = db::get_sidebar_width() {
        ui.set_sidebar_width(width);
    }
    
    ui.on_save_theme_mode(move |mode| {
        if let Err(e) = db::save_theme_mode(mode.as_str()) {
            eprintln!("Failed to save theme mode: {}", e);
        }
    });
    
    if let Ok(mode) = db::get_theme_mode() {
        ui.set_theme_mode(mode.into());
    }

    ui.run()
}
