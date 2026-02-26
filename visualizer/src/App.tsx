import { useState, useEffect, useCallback } from 'react';
import init, { run_simulation_wasm, generate_passengers_wasm, Passenger } from '../pkg/elevator_sim';
import { Play, Pause, SkipBack, SkipForward, RefreshCw, ChevronDown, ChevronUp, Settings2, Copy, Check, RotateCcw } from 'lucide-react';

export default function App() {
  const [seed, setSeed] = useState(0);
  const [output, setOutput] = useState('');
  const [generatedInput, setGeneratedInput] = useState('');
  const [turn, setTurn] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [history, setHistory] = useState<any[]>([]);
  const [isWasmLoaded, setIsWasmLoaded] = useState(false);
  const [showConfig, setShowConfig] = useState(true);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    init().then(() => setIsWasmLoaded(true));
  }, []);

  useEffect(() => {
    if (!isWasmLoaded) return;
    try {
      const n = 10, m = 3, c = 10, t = 100, lambda = 0.1;
      const passengerSource = generate_passengers_wasm(BigInt(seed)) as Passenger[][][];
      let text = `${n} ${m} ${c} ${t} ${lambda}\n`;
      for (let floor = 0; floor < n; floor++) {
        const floorTurns = [];
        for (let trn = 0; trn < t; trn++) {
          const ps = passengerSource[floor][trn];
          floorTurns.push(`${ps.length}${ps.length > 0 ? ' ' : ''}${ps.map(p => p.target_floor).join(' ')}`);
        }
        text += floorTurns.join(' ') + '\n';
      }
      setGeneratedInput(text);
    } catch (e) {
      console.error("Input generation failed", e);
    }
  }, [seed, isWasmLoaded]);

  const copyToClipboard = () => {
    navigator.clipboard.writeText(generatedInput);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const runSimulation = useCallback(async () => {
    if (!isWasmLoaded) return;
    try {
      const snapshots = run_simulation_wasm(BigInt(seed), output) as any[];
      
      const snapshotsWithWait = snapshots.map(s => ({
        ...s,
        elevators: s.elevators.map((el: any) => ({
          ...el,
          passengers: el.passengers.map((p: any) => ({
            ...p,
            waitTime: s.turn - p.arrival_turn
          }))
        })),
        floors: s.floors.map((f: any) => ({
          ...f,
          waiting: f.waiting.map((p: any) => ({
            ...p,
            waitTime: s.turn - p.arrival_turn
          }))
        }))
      }));

      setHistory(snapshotsWithWait);
      setTurn(0);
      setShowConfig(false);
    } catch (e: any) {
      console.error(e);
      alert('Simulation failed: ' + e);
    }
  }, [seed, output, isWasmLoaded]);

  useEffect(() => {
    let interval: any;
    if (isPlaying && turn < history.length - 1) {
      interval = setInterval(() => {
        setTurn(t => t + 1);
      }, 150);
    } else {
      setIsPlaying(false);
    }
    return () => clearInterval(interval);
  }, [isPlaying, turn, history]);

  const currentState = history[turn];

  const getPassengerStyle = (waitTime: number) => {
    const ratio = Math.min(waitTime / 30, 1.0);
    const r = Math.round(251 + (239 - 251) * ratio);
    const g = Math.round(191 + (68 - 191) * ratio);
    const b = Math.round(36 + (68 - 36) * ratio);
    return {
      backgroundColor: `rgb(${r}, ${g}, ${b})`,
      borderColor: `rgba(${r}, ${g}, ${b}, 0.5)`,
      boxShadow: waitTime > 20 ? `0 0 8px rgba(${r}, ${g}, ${b}, 0.4)` : 'none'
    };
  };

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 font-sans selection:bg-blue-500/30 flex flex-col">
      
      <header className="h-16 border-b border-slate-800 bg-slate-900/50 backdrop-blur-md sticky top-0 z-50 px-6 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-4">
          <div className="bg-blue-600 p-1.5 rounded-lg shadow-lg shadow-blue-900/40">
            <Settings2 className="w-5 h-5 text-white" />
          </div>
          <h1 className="text-xl font-bold tracking-tight text-white">Elevator<span className="text-blue-500">Sim</span></h1>
        </div>

        <div className="flex items-center gap-8">
          <div className="hidden md:flex flex-col items-end">
            <span className="text-[10px] text-slate-500 uppercase font-bold tracking-widest leading-none mb-1">Global Score</span>
            <span className="text-xl font-mono font-bold text-green-400 leading-none">
              {currentState?.score.toLocaleString() || 0}
            </span>
          </div>
          <button 
            onClick={() => setShowConfig(!showConfig)}
            className="flex items-center gap-2 px-4 py-2 bg-slate-800 hover:bg-slate-700 rounded-lg border border-slate-700 transition-colors text-sm font-medium"
          >
            Config {showConfig ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
          </button>
        </div>
      </header>

      <main className="p-6 flex-1 overflow-y-auto space-y-6 flex flex-col min-h-0">
        
        {showConfig && (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 bg-slate-900 border border-slate-800 p-6 rounded-2xl shadow-2xl shrink-0 animate-in slide-in-from-top duration-300">
            <div className="space-y-4">
              <h3 className="text-xs font-bold text-slate-400 uppercase tracking-widest flex items-center gap-2">
                <RefreshCw className="w-3.5 h-3.5" /> Simulation Config
              </h3>
              <div className="bg-slate-800/50 p-4 rounded-xl border border-slate-700/50 space-y-4">
                <div>
                  <label className="block text-[10px] font-bold text-slate-500 mb-1.5 uppercase">Input Seed</label>
                  <input 
                    type="number" 
                    min="0"
                    value={seed}
                    onChange={e => setSeed(Math.max(0, parseInt(e.target.value) || 0))}
                    className="w-full bg-slate-950 border border-slate-700 rounded-lg px-4 py-2 focus:ring-2 focus:ring-blue-500 outline-none transition-all font-mono text-sm"
                  />
                </div>
                <button 
                  onClick={copyToClipboard}
                  className="w-full flex items-center justify-center gap-2 py-2.5 bg-slate-700 hover:bg-slate-600 rounded-lg border border-slate-600 transition-all text-xs font-bold active:scale-[0.98]"
                >
                  {copied ? <Check className="w-3.5 h-3.5 text-green-500" /> : <Copy className="w-3.5 h-3.5" />}
                  {copied ? 'Copied!' : `Copy Input (Seed ${seed})`}
                </button>
              </div>
              <button 
                onClick={runSimulation}
                className="w-full bg-blue-600 hover:bg-blue-500 text-white text-sm font-bold py-2.5 rounded-xl transition-all shadow-lg shadow-blue-900/20 active:scale-[0.98]"
              >
                Load & Visualize
              </button>
            </div>

            <div className="space-y-4 flex flex-col">
              <h3 className="text-xs font-bold text-slate-400 uppercase tracking-widest">Agent Output</h3>
              <textarea 
                value={output}
                onChange={e => setOutput(e.target.value)}
                placeholder="Paste your agent's output here..."
                className="w-full flex-1 min-h-[120px] bg-slate-950 border border-slate-700 rounded-xl px-4 py-3 text-xs font-mono focus:ring-2 focus:ring-blue-500 outline-none resize-none transition-all"
              />
            </div>
          </div>
        )}

        <div className="flex-1 bg-slate-900 border border-slate-800 rounded-2xl p-6 shadow-xl relative overflow-hidden flex flex-col min-h-0">
          <div className="absolute top-0 left-0 w-full h-full bg-[radial-gradient(circle_at_50%_50%,rgba(30,41,59,0.3),transparent)] pointer-events-none" />

          <div className="relative flex-1 overflow-y-auto custom-scrollbar">
            <div className="flex justify-center gap-4 min-w-fit pb-4 text-white">
              <div className="flex flex-col-reverse gap-0">
                {[...Array(10)].map((_, floor) => (
                  <div key={floor} className="h-16 flex items-center justify-end pr-4 text-[10px] font-black text-slate-600 border-r border-slate-800 w-10">
                    {floor}F
                  </div>
                ))}
              </div>

              <div className="flex gap-8">
                <div className="flex gap-3">
                  {[0, 1, 2].map(elIdx => {
                    const el = currentState?.elevators[elIdx];
                    const bottomOffset = (el?.floor || 0) * 64;
                    return (
                      <div key={elIdx} className="w-20 h-[640px] bg-slate-950/30 border-x border-slate-800/30 relative shrink-0">
                        <div 
                          className="absolute left-1 w-18 h-14 bg-gradient-to-br from-slate-700 to-slate-800 rounded border border-slate-500 shadow-xl flex flex-col items-center justify-start p-1 transition-all duration-300 ease-in-out z-10"
                          style={{ bottom: `${bottomOffset + 4}px` }}
                        >
                          <div className="text-[7px] font-black text-slate-400 uppercase mb-0.5 leading-none">EL-{elIdx}</div>
                          <div className="grid grid-cols-5 gap-0.5 w-full">
                            {el?.passengers.map((p: any, idx: number) => (
                              <div 
                                key={idx} 
                                style={getPassengerStyle(p.waitTime)}
                                className="w-3 h-4 rounded-[1px] border-[0.5px] flex items-center justify-center transition-colors duration-500"
                              >
                                <span className="text-[6px] font-black text-slate-900 leading-none">{p.target_floor}</span>
                              </div>
                            ))}
                          </div>
                        </div>
                        {[...Array(10)].map((_, f) => (
                          <div key={f} className="absolute w-full border-t border-slate-800/20 h-16" style={{ bottom: `${f * 64}px` }} />
                        ))}
                      </div>
                    );
                  })}
                </div>

                <div className="flex-1 flex flex-col-reverse gap-0 min-w-[400px]">
                  {[...Array(10)].map((_, floor) => {
                    const floorData = currentState?.floors[floor];
                    return (
                      <div key={floor} className="h-16 border-b border-slate-800/30 flex items-center px-4 relative group">
                        <div className="absolute inset-0 bg-blue-500/5 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none" />
                        <div className="flex items-center gap-1 flex-wrap">
                          {floorData?.waiting.map((p: any, idx: number) => (
                            <div 
                              key={idx} 
                              style={getPassengerStyle(p.waitTime)}
                              className="w-5 h-8 rounded-sm flex flex-col items-center justify-end pb-1 shadow-sm border transition-all duration-500 relative group/p"
                            >
                              <span className="text-[7px] font-bold text-slate-900/50 absolute top-0.5 leading-none">{p.waitTime}</span>
                              <span className="text-[9px] font-black text-slate-900 leading-none">{p.target_floor}</span>
                            </div>
                          ))}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-slate-900 border border-slate-800 p-4 rounded-2xl shadow-xl shrink-0">
          <div className="flex items-center gap-6">
            <div className="flex gap-2 shrink-0">
              <button 
                onClick={() => setTurn(0)} 
                className="p-2.5 hover:bg-slate-800 rounded-lg transition-colors text-slate-400 hover:text-white border border-transparent hover:border-slate-700"
                title="Reset to Turn 0"
              >
                <RotateCcw className="w-5 h-5" />
              </button>
              <button 
                onClick={() => setIsPlaying(!isPlaying)} 
                className="w-10 h-10 bg-blue-600 hover:bg-blue-500 rounded-xl flex items-center justify-center shadow-lg transition-all active:scale-90"
              >
                {isPlaying ? <Pause className="w-5 h-5 fill-white" /> : <Play className="w-5 h-5 fill-white translate-x-0.5" />}
              </button>
            </div>

            <div className="flex-1 space-y-1">
              <div className="flex justify-between text-[9px] font-bold text-slate-500 uppercase tracking-wider">
                <span>Turn 0</span>
                <span className="text-blue-500 font-black">Turn {turn}</span>
                <span>Turn 99</span>
              </div>
              <input 
                type="range" 
                min="0" 
                max={Math.max(0, history.length - 1)} 
                value={turn} 
                onChange={e => setTurn(parseInt(e.target.value))}
                className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-blue-500"
              />
            </div>

            <div className="flex gap-1 shrink-0">
              <button 
                onClick={() => setTurn(Math.max(0, turn - 1))} 
                className="p-2.5 hover:bg-slate-800 rounded-lg transition-colors text-slate-400 hover:text-white"
                title="Previous Turn"
              >
                <SkipBack className="w-4 h-4" />
              </button>
              <button 
                onClick={() => setTurn(Math.min(history.length - 1, turn + 1))}
                className="p-2.5 hover:bg-slate-800 rounded-lg transition-colors text-slate-400 hover:text-white"
                title="Next Turn"
              >
                <SkipForward className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      </main>

      <footer className="py-4 text-center text-slate-600 text-[9px] font-medium uppercase tracking-[0.2em] shrink-0">
        Rust (Wasm) + React + Tailwind
      </footer>

      <style>{`
        .custom-scrollbar::-webkit-scrollbar { width: 6px; height: 6px; }
        .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
        .custom-scrollbar::-webkit-scrollbar-thumb { background: #334155; border-radius: 10px; }
        .custom-scrollbar::-webkit-scrollbar-thumb:hover { background: #475569; }
      `}</style>
    </div>
  );
}
