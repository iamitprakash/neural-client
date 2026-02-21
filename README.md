# Neural Mail 🦀✨

A high-performance, privacy-first email client built with Rust and Slint, featuring deep local AI integration.

<img width="4096" height="2522" alt="image" src="https://github.com/user-attachments/assets/4a4e1caa-1cde-44f0-b1ac-cc6669469c3e" />


## Key Features

- **Outlook-Inspired UI**: A modern, responsive design with support for Light and Dark modes, resizable sidebars, and fluid navigation.
- **Tejas AI Assistant**: Chat with your entire 1000+ email inbox using local LLMs. Ask about action items, summarize themes, or find specific invoices without your data ever leaving your machine.
- **Reply w/ AI ✨**: Generate professional, context-aware email drafts instantly based on the active thread.
- **Instant Greetings**: Blazing-fast, time-aware conversational responses for simple greetings, bypassing the LLM for better responsiveness.
- **Advanced Composition**: Support for CC/BCC fields, local attachment management, and integrated email format validation.
- **Privacy First**: All emails are stored in a local SQLite database, and all AI processing is handled locally via Ollama.

## Technology Stack

- **Core**: [Rust](https://www.rust-lang.org/)
- **UI Framework**: [Slint](https://slint.dev/)
- **Database**: [SQLite](https://sqlite.org/) (via rusqlite)
- **Local AI**: [Ollama](https://ollama.com/) (Llama 3.1)
- **Networking**: Reqwest, Tokio

## Getting Started

### Prerequisites

1. **Ollama**: Install [Ollama](https://ollama.com/) and download the Llama 3.1 model:
   ```bash
   ollama run llama3.1
   ```
2. **Rust**: Ensure you have the latest Rust toolchain installed.

### Installation & Run

1. Clone the repository:

   ```bash
   git clone <repo-url>
   cd email-client
   ```

2. Run the application:
   ```bash
   cargo run --release
   ```

## Local Development

The project is structured into three main components:

- `src/main.rs`: Application logic, Slint callbacks, and event loop.
- `src/ai.rs`: Local LLM integration handling 32k context windows for deep email analysis.
- `ui/app.slint`: High-performance UI definitions and layout logic.

---
