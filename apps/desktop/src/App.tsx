import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

function App() {
  const [status, setStatus] = useState("Idle");
  const [isRecording, setIsRecording] = useState(false);
  const [version, setVersion] = useState("0.0.0");
  const [history, setHistory] = useState<string[]>([]);

  useEffect(() => {
    invoke("get_version").then((v) => setVersion(v as string));

    const unlistenResult = listen("transcription-result", (event) => {
      const text = event.payload as string;
      setHistory((prev) => [text, ...prev]);

      // Auto-copy to clipboard
      navigator.clipboard.writeText(text).catch(console.error);

      setStatus("Transcribed & Copied!");
      setTimeout(() => setStatus("Idle"), 2000);
    });

    const unlistenError = listen("transcription-error", (event) => {
      console.error(event.payload);
      setStatus("Error");
      setTimeout(() => setStatus("Idle"), 3000);
    });

    return () => {
      unlistenResult.then((f) => f());
      unlistenError.then((f) => f());
    };
  }, []);

  async function toggleRecording() {
    try {
      const res = await invoke("toggle_recording");
      if (res === "started") {
        setIsRecording(true);
        setStatus("Recording...");
      } else {
        setIsRecording(false);
        setStatus("Transcribing...");
      }
    } catch (e) {
      console.error(e);
      setStatus("Error");
    }
  }

  return (
    <div className="container">
      <div className="mic-wrapper" onClick={toggleRecording}>
        <div className={`mic-button ${isRecording ? 'recording' : ''}`}>
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" /><path d="M19 10v2a7 7 0 0 1-14 0v-2" /><line x1="12" x2="12" y1="19" y2="22" />
          </svg>
        </div>
        {isRecording && <div className="pulse" />}
      </div>

      <div className="status-text">{status}</div>
      <div className="shortcut-hint">Press <b>Ctrl + F12</b> or <b>F8</b> to toggle</div>

      <div className="transcription-history">
        {history.map((text, i) => (
          <div key={i} className="history-item" onClick={() => navigator.clipboard.writeText(text)}>
            <div className="history-text">{text}</div>
            <div className="history-copy-hint">Click to copy</div>
          </div>
        ))}
      </div>

      <div style={{ display: 'flex', gap: '10px', marginTop: 'auto', paddingBottom: '20px' }}>
        <button
          onClick={toggleRecording}
          style={{ padding: '8px 16px', fontSize: '11px', background: '#333', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
        >
          {isRecording ? 'Stop' : 'Start'} (Manual)
        </button>
        <button
          onClick={() => invoke("open_data_folder")}
          style={{ padding: '8px 16px', fontSize: '11px', background: '#333', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
        >
          Open Logs
        </button>
      </div>

      <div className="version-info" style={{ fontSize: '10px', opacity: 0.5, marginBottom: '10px' }}>v{version}</div>
    </div>
  );
}

export default App;
