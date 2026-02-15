import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface EmailHeader {
  id: number;
  subject: string;
  from: string;
  date: string;
}

function App() {
  const [emails, setEmails] = useState<EmailHeader[]>([]);
  const [selectedEmail, setSelectedEmail] = useState<EmailHeader | null>(null);
  const [summary, setSummary] = useState<string>("");
  const [loading, setLoading] = useState(false);

  async function fetchEmails() {
    try {
      setLoading(true);
      // Mocked parameters for demonstration.
      // In a full app, these would come from the account setup form.
      const res = await invoke<EmailHeader[]>("get_emails", {
        email: "test@example.com",
        imapHost: "imap.example.com",
        imapPort: 993,
      });
      setEmails(res);
    } catch (e) {
      console.error(e);
      // For demo purposes, we can populate with mock data if the IMAP connection fails
      setEmails([
        {
          id: 1,
          subject: "Introducing Antigravity Mail",
          from: "welcome@antigravity.io",
          date: "Feb 16",
        },
        {
          id: 2,
          subject: "Quarterly Review 2026",
          from: "boss@startup.com",
          date: "Feb 15",
        },
        {
          id: 3,
          subject: "Local AI is here",
          from: "ollama@community.org",
          date: "Feb 14",
        },
      ]);
    } finally {
      setLoading(false);
    }
  }

  async function summarizeEmail() {
    if (!selectedEmail) return;
    try {
      setLoading(true);
      const res = await invoke<string>("summarize_content", {
        text: `From: ${selectedEmail.from}\nSubject: ${selectedEmail.subject}\n\nThis is the email body content that we want to summarize using our local LLM.`,
      });
      setSummary(res);
    } catch (e) {
      setSummary(
        "Error: Make sure Ollama is running locally with 'llama3' model.",
      );
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    fetchEmails();
  }, []);

  return (
    <div className="app-container">
      {/* Sidebar */}
      <div className="sidebar glass-panel">
        <h2
          style={{
            marginBottom: "30px",
            fontSize: "1.2rem",
            letterSpacing: "1px",
          }}
        >
          ANTIGRAVITY
        </h2>
        <div className="nav-item active">Inbox</div>
        <div className="nav-item">Sent</div>
        <div className="nav-item">Drafts</div>
        <div className="nav-item">Trash</div>

        <div style={{ marginTop: "auto" }}>
          <div
            className="nav-item"
            style={{
              border: "1px dashed var(--glass-border)",
              textAlign: "center",
            }}
          >
            + Add Account
          </div>
        </div>
      </div>

      {/* Email List */}
      <div className="email-list glass-panel">
        <div
          style={{
            padding: "20px",
            borderBottom: "1px solid var(--glass-border)",
          }}
        >
          <input
            type="text"
            placeholder="Search emails..."
            style={{
              width: "100%",
              background: "rgba(255,255,255,0.05)",
              border: "1px solid var(--glass-border)",
              borderRadius: "6px",
              padding: "8px 12px",
              color: "white",
            }}
          />
        </div>
        {emails.map((email) => (
          <div
            key={email.id}
            className={`email-card ${selectedEmail?.id === email.id ? "selected" : ""}`}
            onClick={() => {
              setSelectedEmail(email);
              setSummary("");
            }}
          >
            <div className="subject">
              {email.subject}
              {email.id === 3 && <span className="ai-badge">AI</span>}
            </div>
            <div className="from">{email.from}</div>
          </div>
        ))}
      </div>

      {/* Detail View */}
      <div className="email-detail">
        {selectedEmail ? (
          <div>
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "flex-start",
              }}
            >
              <div>
                <h1 style={{ marginBottom: "10px" }}>
                  {selectedEmail.subject}
                </h1>
                <div style={{ color: "var(--text-secondary)" }}>
                  From:{" "}
                  <span style={{ color: "var(--accent-color)" }}>
                    {selectedEmail.from}
                  </span>
                </div>
              </div>
              <div
                style={{ color: "var(--text-secondary)", fontSize: "0.9rem" }}
              >
                {selectedEmail.date}
              </div>
            </div>

            <button
              className="ai-button"
              onClick={summarizeEmail}
              disabled={loading}
            >
              <span style={{ fontSize: "1.2rem" }}>✦</span>
              {loading ? "Thinking..." : "Summarize with AI"}
            </button>

            {summary && (
              <div
                style={{
                  marginTop: "20px",
                  padding: "20px",
                  background: "rgba(0,122,255,0.1)",
                  borderRadius: "12px",
                  border: "1px solid rgba(0,122,255,0.2)",
                  lineHeight: "1.6",
                }}
              >
                <div
                  style={{
                    fontWeight: "bold",
                    marginBottom: "10px",
                    color: "var(--accent-color)",
                  }}
                >
                  AI SUMMARY
                </div>
                {summary}
              </div>
            )}

            <div
              style={{
                marginTop: "40px",
                fontSize: "1.1rem",
                lineHeight: "1.8",
                color: "rgba(255,255,255,0.8)",
              }}
            >
              <p>Hello,</p>
              <p>
                This is a demonstration of the Antigravity Mail client built
                with Rust and Tauri. The interface is designed to be sleek,
                fast, and AI-native.
              </p>
              <p>
                By clicking the "Summarize with AI" button, you can trigger a
                local LLM (like Ollama) to process your emails without them ever
                leaving your machine.
              </p>
              <p>
                Best regards,
                <br />
                The Antigravity Team
              </p>
            </div>
          </div>
        ) : (
          <div
            style={{
              height: "100%",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              color: "var(--text-secondary)",
            }}
          >
            Select an email to view it
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
