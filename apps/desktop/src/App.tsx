import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [status, setStatus] = useState("Idle");
  const [isRecording, setIsRecording] = useState(false);

  async function toggleRecording() {
    try {
      const res = await invoke("toggle_recording");
      if (res === "started") {
        setIsRecording(true);
        setStatus("Recording...");
      } else {
        setIsRecording(false);
        setStatus("Transcribing...");
        setTimeout(() => setStatus("Idle"), 3000);
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
      <div className="shortcut-hint">Ctrl + Shift + V</div>
    </div>
  );
}

export default App;
