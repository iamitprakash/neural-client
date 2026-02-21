mod auth;
mod mail;
mod ai;
mod db;

slint::include_modules!();

use slint::{Model, ModelRc, VecModel};
use chrono::Timelike;
use std::rc::Rc;
use tokio::runtime::Runtime;

use rand::RngExt;
use email_address::EmailAddress;
use std::env;
use tracing::{info, error, warn};
use tracing_subscriber;

fn sanitize_for_prompt(text: &str) -> String {
    // Basic sanitization to prevent prompt injection
    // Escape characters that models often use for structure
    text.replace('{', "(")
        .replace('}', ")")
        .replace('[', "(")
        .replace(']', ")")
        .replace("---", " - ")
        .replace("###", " # ")
}

fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();
    info!("Starting Neural Mail Client...");
    let ui = AppWindow::new()?;
    let rt = Runtime::new().unwrap();
    let rt_handle_fetch = rt.handle().clone();
    let rt_handle_email_chat = rt.handle().clone();
    let rt_handle_chat = rt.handle().clone();
    let rt_handle_reply = rt.handle().clone();
    
    // Initialize both DBs (accounts if we ever use them, and emails)
    let _ = auth::init_db();
    let _ = db::init_db();

    // Check for Master Password
    match db::get_master_password_hash() {
        Ok(Some(_)) => ui.set_has_master_password(true),
        _ => ui.set_has_master_password(false),
    }
    
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
                    category: "Inbox".into(),
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
                        category: e.category.into(),
                    });
                }
                let model = Rc::new(VecModel::from(slint_emails));
                ui.set_emails(ModelRc::from(model));
                ui.set_status_message("Loaded Emails from DB".into());
                
                // Background Categorize Task
                let rt_bg = rt_handle_fetch.clone();
                rt_bg.spawn(async move {
                    if let Ok(emails) = db::get_all_emails() {
                        for e in emails.iter().take(20) { // Just categorize first 20 for demo
                            if e.category == "Inbox" {
                                if let Ok(new_cat) = ai::categorize_email(&e.subject, &e.body).await {
                                    let _ = db::update_email_category(e.id, &new_cat);
                                }
                            }
                        }
                    }
                });
            },
            Err(e) => {
                ui.set_status_message(format!("DB error: {}", e).into());
            }
        }
    });

    let ui_handle_cat = ui.as_weak();
    ui.on_category_changed(move |cat: slint::SharedString| {
        if let Some(ui) = ui_handle_cat.upgrade() {
            let mut slint_emails = Vec::new();
            match db::get_emails_by_category(cat.as_str()) {
                Ok(db_emails) => {
                    for e in db_emails {
                        slint_emails.push(Email {
                            id: e.id,
                            subject: e.subject.into(),
                            sender: e.sender.into(),
                            date: e.date.into(),
                            body: e.body.into(),
                            has_attachment: e.has_attachment,
                            category: e.category.into(),
                        });
                    }
                    ui.set_emails(ModelRc::from(Rc::new(VecModel::from(slint_emails))));
                }
                Err(e) => eprintln!("Category error: {}", e),
            }
        }
    });

    let ui_handle_search = ui.as_weak();
    ui.on_search_changed(move |query: slint::SharedString| {
        if let Some(ui) = ui_handle_search.upgrade() {
            let mut slint_emails = Vec::new();
            match db::search_emails(query.as_str()) {
                Ok(db_emails) => {
                    for e in db_emails {
                        slint_emails.push(Email {
                            id: e.id,
                            subject: e.subject.into(),
                            sender: e.sender.into(),
                            date: e.date.into(),
                            body: e.body.into(),
                            has_attachment: e.has_attachment,
                            category: e.category.into(),
                        });
                    }
                    ui.set_emails(ModelRc::from(Rc::new(VecModel::from(slint_emails))));
                }
                Err(e) => eprintln!("Search error: {}", e),
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

        rt_handle_email_chat.spawn(async move {
            let s_sender = sanitize_for_prompt(&sender);
            let s_subject = sanitize_for_prompt(&subject);
            let s_body = sanitize_for_prompt(&body);
            let s_msg = sanitize_for_prompt(&msg_clone);

            let context_str = format!("From: {}\nSubject: {}\nBody: {}\n", s_sender, s_subject, s_body);
            
            let result = ai::chat_with_emails(&s_msg, &context_str).await;
            
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

        // Instant Greeting Interceptor
        let normalized = msg_clone.trim().to_lowercase();
        let greetings = ["hi", "hello", "hey", "good morning", "good evening", "greetings", "hi!", "hello!", "hey!"];
        
        if greetings.contains(&normalized.as_str()) {
            let hour = chrono::Local::now().hour();
            let time_greeting = if hour < 12 {
                "Good morning"
            } else if hour < 17 {
                "Good afternoon"
            } else {
                "Good evening"
            };
            
            let reply = format!("Hello! {}, how may I help you today?", time_greeting);
            
            let mut history: Vec<ChatMessage> = ui.get_chat_history().iter().collect();
            history.push(ChatMessage {
                is_user: false,
                text: reply.into(),
            });
            ui.set_chat_history(ModelRc::from(Rc::new(VecModel::from(history))));
            ui.set_loading(false);
            return;
        }

        rt_handle_chat.spawn(async move {
            let mut context_str = String::new();
            if let Ok(emails) = db::get_all_emails() {
                for e in emails.iter().take(100) {
                    let s_sender = sanitize_for_prompt(&e.sender);
                    let s_subject = sanitize_for_prompt(&e.subject);
                    let s_body = sanitize_for_prompt(&e.body);
                    context_str.push_str(&format!("From: {}, Subject: {}\nBody: {}\n\n", s_sender, s_subject, s_body));
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

    let _ui_handle_sidebar = ui.as_weak();
    ui.on_save_sidebar_width(move |width| {
        if let Err(e) = db::save_sidebar_width(width) {
            error!("Failed to save sidebar width: {}", e);
        }
    });

    if let Ok(width) = db::get_sidebar_width() {
        ui.set_sidebar_width(width);
    }
    
    ui.on_save_theme_mode(move |mode| {
        if let Err(e) = db::save_theme_mode(mode.as_str()) {
            error!("Failed to save theme mode: {}", e);
        }
    });
    
    if let Ok(mode) = db::get_theme_mode() {
        ui.set_theme_mode(mode.into());
    }

    let ui_handle_send = ui.as_weak();
    ui.on_send_email(move |to, cc, bcc, subject, body, attachments, force_send| {
        let to_str = to.to_string();
        let cc_str = cc.to_string();
        let bcc_str = bcc.to_string();
        let body_str = body.to_string();

        fn is_valid_email(s: &str) -> bool {
            let trimmed = s.trim();
            if trimmed.is_empty() { return true; } // Optional fields can be empty
            EmailAddress::is_valid(trimmed)
        }

        if to_str.trim().is_empty() {
            if let Some(ui) = ui_handle_send.upgrade() {
                ui.set_compose_error("Please enter a recipient in the 'To' field.".into());
            }
            return;
        }

        if !is_valid_email(&to_str) {
            if let Some(ui) = ui_handle_send.upgrade() {
                ui.set_compose_error("Invalid email format in the 'To' field.".into());
            }
            return;
        }

        if !is_valid_email(&cc_str) {
            if let Some(ui) = ui_handle_send.upgrade() {
                ui.set_compose_error("Invalid email format in the 'Cc' field.".into());
            }
            return;
        }

        if !is_valid_email(&bcc_str) {
            if let Some(ui) = ui_handle_send.upgrade() {
                ui.set_compose_error("Invalid email format in the 'Bcc' field.".into());
            }
            return;
        }

        // --- Heuristic Warning Checks ---
        if !force_send {
            // Isolate user draft from previous emails
            let drafts: Vec<&str> = body_str.split("--- Original Message ---").collect();
            let user_draft = drafts[0].to_lowercase();

            // 1. Missing Attachment Check
            let att_keywords = ["attach", "attached", "attachment", "enclosed"];
            let mut mentions_attachment = false;
            for kw in att_keywords.iter() {
                if user_draft.contains(kw) {
                    mentions_attachment = true;
                    break;
                }
            }
            if mentions_attachment && attachments.row_count() == 0 {
                if let Some(ui) = ui_handle_send.upgrade() {
                    ui.set_compose_warning("It seems you mentioned an attachment but forgot to add one. Click Send again to ignore.".into());
                    ui.set_force_send(true);
                }
                return;
            }

            // 2. Missing Link Check
            if user_draft.contains("link") {
                if !user_draft.contains("http://") && !user_draft.contains("https://") && !user_draft.contains("www.") {
                    if let Some(ui) = ui_handle_send.upgrade() {
                        ui.set_compose_warning("It seems you mentioned a link but forgot to include the URL. Click Send again to ignore.".into());
                        ui.set_force_send(true);
                    }
                    return;
                }
            }
        }

        if env::var("DEV_MODE").unwrap_or_else(|_| "false".to_string()) == "true" {
            info!("====== OUTBOUND EMAIL MOCK ======");
            info!("TO: {}", to_str);
            info!("CC: {}", cc_str);
            info!("BCC: {}", bcc_str);
            info!("SUBJECT: {}", subject);
            info!("ATTACHMENTS: {} files", attachments.row_count());
            for i in 0..attachments.row_count() {
                info!(" - {}", attachments.row_data(i).unwrap());
            }
            info!("------------- BODY --------------");
            info!("{}", body);
            info!("=================================");
        } else {
            // Future: Implement real secure SMTP here
            warn!("Production Mode: Real email sending not yet implemented.");
        }
        if let Some(ui) = ui_handle_send.upgrade() {
            ui.set_show_compose_dialog(false);
            ui.set_compose_error("".into());
            ui.set_compose_warning("".into());
            ui.set_force_send(false);
            ui.set_compose_to("".into());
            ui.set_compose_cc("".into());
            ui.set_compose_bcc("".into());
            ui.set_show_cc_bcc(false);
            ui.set_compose_subject("".into());
            ui.set_compose_body("".into());
            
            // clear attachments
            let empty: Vec<slint::SharedString> = Vec::new();
            ui.set_compose_attachments(ModelRc::from(Rc::new(VecModel::from(empty))));
        }
    });

    let ui_handle_attachments = ui.as_weak();
    ui.on_add_attachment(move || {
        if let Some(ui) = ui_handle_attachments.upgrade() {
            if let Some(file_path) = rfd::FileDialog::new()
                .set_title("Select Attachment")
                .pick_file() 
            {
                let filename = file_path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                let mut current: Vec<slint::SharedString> = ui.get_compose_attachments().iter().collect();
                current.push(filename.into());
                ui.set_compose_attachments(ModelRc::from(Rc::new(VecModel::from(current))));
            }
        }
    });

    let ui_handle_remove = ui.as_weak();
    ui.on_remove_attachment(move |idx| {
        if let Some(ui) = ui_handle_remove.upgrade() {
            let mut current: Vec<slint::SharedString> = ui.get_compose_attachments().iter().collect();
            if (idx as usize) < current.len() {
                current.remove(idx as usize);
                ui.set_compose_attachments(ModelRc::from(Rc::new(VecModel::from(current))));
            }
        }
    });

    let ui_handle_ai_reply = ui.as_weak();
    ui.on_generate_ai_reply(move |sender, subject, original_body| {
        let ui_handle_async = ui_handle_ai_reply.clone();
        let sender_clone = sender.to_string();
        let subject_clone = subject.to_string();
        let original_body_clone = original_body.to_string();
        
        rt_handle_reply.spawn(async move {
            let result = ai::generate_reply(&original_body_clone).await;
            
            let final_reply = match result {
                Ok(reply) => reply,
                Err(e) => format!("Error generating reply: {}", e),
            };
            
            // Reconstruct the full compose body with the AI's draft on top
            let reconstructed_body = format!(
                "{}\n\n--- Original Message ---\nFrom: {}\nSubject: {}\n\n{}", 
                final_reply, sender_clone, subject_clone, original_body_clone
            );
            
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle_async.upgrade() {
                    ui.set_compose_body(reconstructed_body.into());
                }
            });
        });
    });

    let ui_handle_auth = ui.as_weak();
    ui.on_create_master_password(move |password| {
        let ui = ui_handle_auth.unwrap();
        let pass_str = password.to_string();
        if pass_str.len() < 4 {
            ui.set_status_message("Password too short (min 4 chars)".into());
            return;
        }
        
        let hash = bcrypt::hash(pass_str, bcrypt::DEFAULT_COST).unwrap();
        if let Ok(_) = db::set_master_password(&hash) {
            ui.set_has_master_password(true);
            ui.set_is_locked(false);
            ui.set_password_input("".into());
            ui.set_status_message("Master password set successfully".into());
        }
    });

    let ui_handle_verify = ui.as_weak();
    ui.on_verify_password(move |password| {
        let ui = ui_handle_verify.unwrap();
        match db::get_master_password_hash() {
            Ok(Some(hash)) => {
                if bcrypt::verify(password.to_string(), &hash).unwrap_or(false) {
                    ui.set_is_locked(false);
                    ui.set_password_input("".into());
                    ui.set_status_message("Unlocked".into());
                } else {
                    ui.set_status_message("Incorrect password".into());
                }
            }
            _ => ui.set_status_message("No master password set".into()),
        }
    });

    ui.run()
}
