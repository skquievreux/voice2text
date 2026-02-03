import React from 'react';

export default function LandingPage() {
  return (
    <main className="min-h-screen bg-[#0a0a0a] text-white selection:bg-purple-500/30 overflow-hidden relative">
      {/* Background Glow */}
      <div className="absolute top-0 left-0 w-full h-full overflow-hidden pointer-events-none">
        <div className="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] bg-purple-600/20 blur-[120px] rounded-full animate-pulse" />
        <div className="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] bg-blue-600/20 blur-[120px] rounded-full animate-pulse delay-1000" />
      </div>

      <nav className="border-b border-white/5 backdrop-blur-md sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 bg-gradient-to-tr from-purple-600 to-blue-500 rounded-lg flex items-center justify-center shadow-lg shadow-purple-500/20">
              <span className="font-bold text-lg">V</span>
            </div>
            <span className="font-bold tracking-tight text-xl">Voice2Text</span>
          </div>
          <div className="flex items-center gap-6 text-sm text-gray-400 font-medium">
            <a href="#" className="hover:text-white transition-colors">Pricing</a>
            <a href="#" className="hover:text-white transition-colors">Docs</a>
            <button className="bg-white text-black px-4 py-2 rounded-full hover:bg-gray-200 transition-all font-semibold">
              Get Started
            </button>
          </div>
        </div>
      </nav>

      <section className="relative pt-32 pb-20 px-6">
        <div className="max-w-4xl mx-auto text-center">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-white/5 border border-white/10 text-xs font-medium text-purple-400 mb-8 animate-fade-in-up">
            <span className="w-2 h-2 rounded-full bg-purple-500 animate-ping" />
            Now Powered by Deepgram Nova-2
          </div>
          <h1 className="text-6xl md:text-8xl font-black tracking-tight mb-8 bg-gradient-to-b from-white to-white/50 bg-clip-text text-transparent leading-[1.1]">
            Speak your ideas. <br />
            <span className="text-purple-500">Instantly</span> typed.
          </h1>
          <p className="text-xl text-gray-400 mb-12 max-w-2xl mx-auto leading-relaxed">
            The invisible desktop layer that transcribes your voice directly into any application with <b>99% accuracy</b> and zero latency.
          </p>
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <button className="w-full sm:w-auto px-8 py-4 bg-purple-600 hover:bg-purple-500 text-white rounded-2xl font-bold text-lg transition-all shadow-xl shadow-purple-600/20 active:scale-95">
              Download for Windows
            </button>
            <button className="w-full sm:w-auto px-8 py-4 bg-white/5 hover:bg-white/10 text-white rounded-2xl font-bold text-lg border border-white/10 transition-all active:scale-95">
              View Demo
            </button>
          </div>
        </div>
      </section>

      {/* Feature Cards */}
      <section className="max-w-7xl mx-auto px-6 py-20 grid grid-cols-1 md:grid-cols-3 gap-8">
        {[
          { title: "Edge Performance", desc: "Edge functions ensure < 200ms round-trips from anywhere in the world.", icon: "⚡" },
          { title: "Zero Installer", desc: "Ultra-lightweight Tauri client (~8MB). Minimal CPU and RAM footprint.", icon: "🪶" },
          { title: "Deepgram API", desc: "Powered by Nova-2 for industry-leading speed and German accuracy.", icon: "🎯" }
        ].map((feature, i) => (
          <div key={i} className="group p-8 rounded-3xl bg-white/[0.02] border border-white/5 hover:border-purple-500/30 hover:bg-white/[0.04] transition-all duration-500">
            <div className="text-4xl mb-6">{feature.icon}</div>
            <h3 className="text-xl font-bold mb-3">{feature.title}</h3>
            <p className="text-gray-400 leading-relaxed text-sm">
              {feature.desc}
            </p>
          </div>
        ))}
      </section>
    </main>
  );
}
