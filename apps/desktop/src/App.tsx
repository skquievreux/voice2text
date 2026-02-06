import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

function App() {
  const [view, setView] = useState<"home" | "settings" | "history">("home");
  const [status, setStatus] = useState("Idle");
  const [version, setVersion] = useState("0.0.0");
  const [lastText, setLastText] = useState("");
  const [campaigns, setCampaigns] = useState<any[]>([]);

  useEffect(() => {
    invoke("get_version").then((v) => setVersion(v as string));
    fetchCampaigns();
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
      setStatus("Error: " + event.payload);
      setTimeout(() => setStatus("Idle"), 5000);
    });

    const unlistenState = listen("recording-state", (event) => {
      const isRecording = event.payload as boolean;
      setStatus(isRecording ? "Recording..." : "Idle");
    });

    return () => {
      clearInterval(interval);
      unlistenResult.then((f) => f());
      unlistenError.then((f) => f());
      unlistenState.then((f) => f());
    };
  }, []);

  const fetchCampaigns = async () => {
    try {
      const result: any = await invoke("fetch_campaigns");
      if (result && result.campaigns) {
        setCampaigns(result.campaigns);
      }
    } catch (err) {
      console.error("Failed to fetch campaigns", err);
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

  if (view === "settings") {
    return (
      <div className="container">
        <SettingsView onBack={() => setView("home")} version={version} />
      </div>
    );
  }

  return (
    <div className="container centered">
      <div style={{ position: "absolute", top: "20px", left: "20px", display: "flex", gap: "10px" }}>
        <HistoryButton onClick={() => setView("history")} />
      </div>

      <button
        className="back-btn"
        onClick={() => setView("settings")}
        style={{ position: "absolute", top: "20px", right: "20px" }}
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1Z" />
        </svg>
      </button>

      {view === "history" && <HistoryView onBack={() => setView("home")} />}

      <div
        className={`mic-button-static ${status === "Recording..." ? "recording" : ""}`}
        onClick={handleToggle}
        style={{ cursor: "pointer", marginBottom: "20px" }}
      >
        {status === "Recording..." && <div className="pulse" />}
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
          <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
          <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
          <line x1="12" x2="12" y1="19" y2="22" />
        </svg>
      </div>

      <div className="status-text" style={{ fontSize: "18px", fontWeight: 600, marginBottom: "10px" }}>{status}</div>

      {(lastText || status.includes("Copied") || status === "No speech detected.") && (
        <div style={{ margin: "15px 0", padding: "15px", background: "rgba(30, 30, 35, 0.95)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: "12px", fontSize: "13px", lineHeight: "1.4", maxWidth: "300px", wordBreak: "break-word", color: "#ffffff", boxShadow: "0 10px 15px -3px rgba(0, 0, 0, 0.5)" }}>
          {lastText || (status === "No speech detected." ? "No speech detected" : "Check clipboard")}
        </div>
      )}

      <div className="shortcut-hint">
        <b>F8</b> or <b>Ctrl+F12</b> to toggle
      </div>

      <div style={{ marginTop: "30px", width: "100%", maxWidth: "320px" }}>
        <div style={{ fontSize: "11px", fontWeight: "800", textTransform: "uppercase", color: "#9ca3af", textAlign: "left", marginBottom: "12px", letterSpacing: "0.1em" }}>Latest News</div>
        {campaigns.length === 0 ? (
          <div style={{ fontSize: "11px", opacity: 0.4 }}>No active campaigns</div>
        ) : (
          <div className="campaign-list" style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
            {campaigns.map((c: any) => (
              <div key={c.id} className="settings-card" style={{ padding: "12px" }}>
                <div style={{ fontWeight: "bold", fontSize: "13px", color: "#818cf8", marginBottom: "4px" }}>{c.title}</div>
                <div style={{ fontSize: "11px", color: "#d1d5db", marginBottom: "8px", textAlign: "left" }}>{c.body}</div>
                {c.cta && (
                  <button onClick={() => invoke("open_browser", { url: c.cta })} style={{ fontSize: "10px", background: "#4f46e5", border: "none", borderRadius: "6px", padding: "6px 10px", color: "white", cursor: "pointer", alignSelf: "flex-start", fontWeight: "700" }}>
                    Details →
                  </button>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="version-info" style={{ position: "absolute", bottom: "15px", opacity: 1, fontSize: "12px", color: "#818cf8", fontWeight: 700 }}>v{version}</div>
    </div>
  );
}

function HistoryButton({ onClick }: { onClick: () => void }) {
  return (
    <button className="back-btn" onClick={onClick}>
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" /></svg>
    </button>
  );
}

function HistoryView({ onBack }: { onBack: () => void }) {
  const [items, setItems] = useState<any[]>([]);
  const [search, setSearch] = useState("");

  useEffect(() => {
    loadHistory();
  }, [search]);

  const loadHistory = () => {
    invoke("get_history", { limit: 50, offset: 0, search: search || null })
      .then((res: any) => setItems(res))
      .catch(console.error);
  };

  const clearHistory = () => {
    if (confirm("Clear all history?")) {
      invoke("clear_all_history").then(() => setItems([]));
    }
  };

  return (
    <div className="settings-container" style={{ position: "fixed", top: 0, left: 0, right: 0, bottom: 0, zIndex: 100, background: "#0a0a0c" }}>
      <div className="settings-header">
        <button className="back-btn" onClick={onBack}>
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="m15 18-6-6 6-6" /></svg>
        </button>
        <h2 style={{ margin: 0, fontSize: "22px", fontWeight: 800 }}>History</h2>
      </div>

      <input
        type="text"
        placeholder="Search past transcriptions..."
        value={search}
        onChange={(e) => setSearch(e.target.value)}
        style={{ width: "100%", background: "#18181b", color: "white", border: "1px solid rgba(255,255,255,0.1)", padding: "12px", borderRadius: "8px", marginBottom: "15px" }}
      />

      <div style={{ flex: 1, overflowY: "auto", display: "flex", flexDirection: "column", gap: "10px" }}>
        {items.length === 0 && <div style={{ opacity: 0.4, textAlign: "center", marginTop: "20px" }}>No history found</div>}
        {items.map((item) => (
          <div key={item.id} className="settings-card" style={{ padding: "12px", textAlign: "left" }}>
            <div style={{ fontSize: "10px", color: "#9ca3af", marginBottom: "5px" }}>{new Date(item.timestamp).toLocaleString()}</div>
            <div style={{ fontSize: "13px", color: "#e5e7eb", marginBottom: "10px", wordBreak: "break-word" }}>{item.text}</div>
            <button
              className="btn-secondary"
              style={{ padding: "4px 8px", fontSize: "10px" }}
              onClick={() => navigator.clipboard.writeText(item.text)}
            >
              COPY
            </button>
          </div>
        ))}
      </div>

      <button className="btn-secondary" style={{ marginTop: "20px", color: "#ef4444" }} onClick={clearHistory}>
        Clear All History
      </button>
    </div>
  );
}

function SettingsView({ onBack, version }: { onBack: () => void; version: string }) {
  const [clientStatus, setClientStatus] = useState<any>(null);
  const [hwId, setHwId] = useState("");
  const [devices, setDevices] = useState<string[]>([]);
  const [selectedDevice, setSelectedDevice] = useState("Default");

  useEffect(() => {
    // Initial Fetch
    invoke("get_client_status").then(setClientStatus);
    invoke("get_hw_id").then((id: any) => setHwId(id as string));

    // Fetch Devices & Active Selection
    Promise.all([
      invoke("get_input_devices"),
      invoke("get_input_device")
    ]).then(([d, active]) => {
      setDevices(["Default", ...(d as string[])]);
      if (active) {
        setSelectedDevice(active as string);
      }
    }).catch(e => console.error("Device Fetch Error", e));

    // Polling Status (Every 60s)
    const interval = setInterval(() => {
      invoke("refresh_client_status").then((s) => {
        setClientStatus(s);
      }).catch(err => console.error("Poll Error:", err));
    }, 60000);

    return () => clearInterval(interval);
  }, []);

  const handleDeviceChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const dev = e.target.value;
    setSelectedDevice(dev);
    invoke("set_input_device", { name: dev });
  };

  const openLogs = () => invoke("open_data_folder");
  const manageSubscription = () => invoke("open_browser", { url: "https://voice2text.runitfast.xyz/dashboard" });

  const formatDate = (dateStr: string) => {
    if (!dateStr) return "N/A";
    return new Date(dateStr).toLocaleDateString();
  };

  return (
    <div className="settings-container">
      <div className="settings-header">
        <button className="back-btn" onClick={onBack}>
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="m15 18-6-6 6-6" /></svg>
        </button>
        <h2 style={{ margin: 0, fontSize: "22px", fontWeight: 800, letterSpacing: "-0.02em" }}>Settings</h2>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: "15px" }}>
        {(clientStatus?.name || clientStatus?.email) && (
          <div className="settings-card">
            <div className="card-label">User Account</div>
            {clientStatus?.name && <div className="card-value" style={{ fontSize: "16px", marginBottom: "2px" }}>{clientStatus.name}</div>}
            {clientStatus?.email && <div style={{ fontSize: "12px", opacity: 0.6 }}>{clientStatus.email}</div>}
          </div>
        )}

        <div className="settings-card">
          <div className="card-label">Subscription Status</div>
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            <div className="card-value" style={{ textTransform: "capitalize", color: clientStatus?.status === "active" ? "#4ade80" : "#fbbf24" }}>
              {clientStatus?.status || "Checking..."}
            </div>
            {clientStatus?.status === "active" && <div style={{ fontSize: "11px", opacity: 0.4 }}>(Ends: {formatDate(clientStatus?.valid_until)})</div>}
          </div>
        </div>
      </div>

      <div className="settings-card">
        <div className="card-label">Device Identifier</div>
        <div className="device-id-box">
          <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: "250px" }}>
            {hwId || "Loading..."}
          </span>
          <button
            onClick={() => navigator.clipboard.writeText(hwId)}
            style={{ background: "transparent", border: "none", color: "#818cf8", fontSize: "11px", fontWeight: 700, cursor: "pointer", marginLeft: "10px" }}
          >
            COPY
          </button>
        </div>
      </div>

      <div className="settings-card">
        <div className="card-label">Microphone Input</div>
        <select
          value={selectedDevice}
          onChange={handleDeviceChange}
          style={{ width: "100%", background: "#000", color: "white", padding: "8px", borderRadius: "6px", border: "1px solid rgba(255,255,255,0.2)", marginTop: "5px" }}
        >
          {devices.map((d, i) => (
            <option key={i} value={d}>{d}</option>
          ))}
        </select>
      </div>

      <div style={{ marginTop: "12px", display: "flex", flexDirection: "column", gap: "12px" }}>
        <button className="btn-primary" onClick={manageSubscription}>
          Manage Subscription
        </button>
        <button className="btn-secondary" onClick={openLogs}>
          Open Logs Folder
        </button>
      </div>

      <div style={{ marginTop: "30px", fontSize: "12px", color: "#818cf8", textAlign: "center", fontWeight: 700 }}>
        Voice2Text Desktop v{version}
      </div>
    </div>
  );
}

export default App;
