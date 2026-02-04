import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

function App() {
  const [status, setStatus] = useState("Idle");
  // const [isRecording, setIsRecording] = useState(false);
  const [version, setVersion] = useState("0.0.0");

  useEffect(() => {
    invoke("get_version").then((v) => setVersion(v as string));

    const unlistenResult = listen("transcription-result", (event) => {
      const text = event.payload as string;
      navigator.clipboard.writeText(text).catch(console.error);
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

  return (
    <div className="container" style={{ padding: '40px 20px', textAlign: 'center' }}>
      <div className={`mic-button-static`} style={{ margin: '0 auto 20px' }}>
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" /><path d="M19 10v2a7 7 0 0 1-14 0v-2" /><line x1="12" x2="12" y1="19" y2="22" />
        </svg>
      </div>

      <div className="status-text" style={{ fontSize: '18px', fontWeight: 600, marginBottom: '10px' }}>{status}</div>
      <div className="shortcut-hint" style={{ opacity: 0.5, fontSize: '12px' }}>
        Status: Background Active<br />
        Shortcut: <b>F8</b> or <b>Ctrl+F12</b>
      </div>

      <div className="version-info" style={{ position: 'absolute', bottom: '15px', width: '100%', fontSize: '10px', opacity: 0.3 }}>v{version}</div>
    </div>
  );
}

export default App;
