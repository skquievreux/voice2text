import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

function App() {
  const [status, setStatus] = useState("Idle");
  // const [isRecording, setIsRecording] = useState(false);
  const [version, setVersion] = useState("0.0.0");
  const [lastText, setLastText] = useState("");

  const [campaigns, setCampaigns] = useState<any[]>([]);

  useEffect(() => {
    invoke("get_version").then((v) => setVersion(v as string));

    // Initial Fetch
    fetchCampaigns();

    // Poll every 5 minutes
    const interval = setInterval(fetchCampaigns, 5 * 60 * 1000);

    const unlistenResult = listen("transcription-result", (event) => {
      const text = event.payload as string;
      if (!text || text.trim() === "") {
        setStatus("No speech detected.");
        setTimeout(() => setStatus("Idle"), 3000);
        return;
      }
      navigator.clipboard.writeText(text).catch(console.error);
      setLastText(text);
      setStatus("Copied to Clipboard!");
      setTimeout(() => setStatus("Idle"), 3000);
    });

    const unlistenError = listen("transcription-error", (event) => {
      console.error(event.payload);
      setStatus("Error: " + event.payload);
      setTimeout(() => setStatus("Idle"), 5000);
    });

    return () => {
      unlistenResult.then((f) => f());
      unlistenError.then((f) => f());
    };
  }, []);

  const fetchCampaigns = async () => {
    try {
      // In Dev: http://localhost:3000, Prod: https://voice2text.runitfast.xyz
      // Check if we are in dev mode (localhost:1420)
      const isDev = window.location.hostname === 'localhost';
      const apiBase = isDev ? 'http://localhost:3000' : 'https://voice2text.runitfast.xyz';

      // We need the token. Ideally the Rust backend passes it, or we fetch it via invoke.
      // For simplicity, we'll try to fetch without token first (if API allows) OR invoke a command to get the token.
      // But the API req requires a token for context. 
      // Let's create a Rust command 'get_token' or 'fetch_campaigns_proxy'.
      // UPDATE: Let's use invoke('fetch_campaigns') to keep auth logic in Rust.

      const result: any = await invoke('fetch_campaigns'); // Needs implementation in Rust
      if (result && result.campaigns) {
        setCampaigns(result.campaigns);
      }
    } catch (err) {
      console.error("Failed to fetch campaigns", err);
      setStatus("CampErr: " + err);
    }
  };

  const handleToggle = async () => {
    try {
      const result = await invoke("toggle_recording");
      setStatus(result === "started" ? "Recording..." : "Idle");
    } catch (err) {
      setStatus("Error: " + err);
    }
  };

  return (
    <div className="container" style={{ padding: '40px 20px', textAlign: 'center' }}>
      <div
        className={`mic-button-static ${status === 'Recording...' ? 'pulse' : ''}`}
        onClick={handleToggle}
        style={{ margin: '0 auto 20px', cursor: 'pointer' }}
      >
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" /><path d="M19 10v2a7 7 0 0 1-14 0v-2" /><line x1="12" x2="12" y1="19" y2="22" />
        </svg>
      </div>

      <div className="status-text" style={{ fontSize: '18px', fontWeight: 600, marginBottom: '10px' }}>{status}</div>

      {/* Transcription Result Display */}
      {(lastText || status.includes("Copied") || status === "No speech detected.") && (
        <div style={{ margin: '15px 0', padding: '15px', background: 'rgba(30, 30, 35, 0.95)', border: '1px solid rgba(255,255,255,0.2)', borderRadius: '8px', fontSize: '13px', lineHeight: '1.4', maxWidth: '300px', wordBreak: 'break-word', opacity: 1, color: '#ffffff', boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.5)' }}>
          {lastText || (status === "No speech detected." ? "No speech detected" : "Check clipboard")}
        </div>
      )}

      <div className="shortcut-hint" style={{ opacity: 0.5, fontSize: '12px' }}>
        Status: Background Active<br />
        Shortcut: <b>F8</b> or <b>Ctrl+F12</b>
      </div>

      {/* Campaigns Section */}
      <div style={{ marginTop: '20px', borderTop: '1px solid rgba(255,255,255,0.1)', paddingTop: '10px' }}>
        <div style={{ fontSize: '12px', fontWeight: 'bold', marginBottom: '8px' }}>Campaigns</div>

        {campaigns.length === 0 ? (
          <div id="campaign-container" style={{ fontSize: '11px', opacity: 0.6 }}>
            {status.includes("CampErr") ? "Campaigns: Network Error" : "No active campaigns"}
          </div>
        ) : (
          <div className="campaign-list" style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {campaigns.map((c: any) => (
              <div key={c.id} style={{ background: 'linear-gradient(90deg, rgba(79, 70, 229, 0.1) 0%, rgba(79, 70, 229, 0.05) 100%)', border: '1px solid rgba(79, 70, 229, 0.3)', borderRadius: '6px', padding: '10px', textAlign: 'left' }}>
                <div style={{ fontWeight: 'bold', fontSize: '12px', color: '#818cf8', marginBottom: '4px' }}>{c.title}</div>
                <div style={{ fontSize: '11px', opacity: 0.8, marginBottom: '6px' }}>{c.body}</div>
                {c.cta && (
                  <button onClick={() => invoke('open_browser', { url: c.cta })} style={{ fontSize: '10px', background: '#4f46e5', border: 'none', borderRadius: '4px', padding: '4px 8px', color: 'white', cursor: 'pointer' }}>
                    Open Offer
                  </button>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="version-info" style={{ position: 'absolute', bottom: '15px', width: '100%', fontSize: '10px', opacity: 0.3 }}>v{version}</div>
    </div>
  );
}

export default App;
