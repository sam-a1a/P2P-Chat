import { useEffect, useState, useRef } from 'react';

// Dynamic import holder – will contain all WASM exports after init
let wasmModule: any = null;

const DESKTOP_WEBRTC_ADDR =
    '/ip4/192.168.1.6/udp/9090/webrtc-direct/certhash/uEiDyoeV-gNjvXu6HusX7VBd1SX-9eMyEWH3Rm1SfHEvcoQ';

type Message = {
  type: 'message' | 'connected' | 'disconnected';
  from?: string;
  text?: string;
  peer?: string;
};

function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [ready, setReady] = useState(false);
  const intervalRef = useRef<number | null>(null);

  useEffect(() => {
    async function start() {
      try {
        // 1. Import the WASM module (gives us the default init + all named exports)
        wasmModule = await import('./wasm/web_wasm.js');

        // 2. Initialize the WASM (call the default export)
        await wasmModule.default(); // this is the equivalent of init()

        // 3. Now all functions are available
        await wasmModule.start_node(DESKTOP_WEBRTC_ADDR);
        wasmModule.subscribe('chat');
        setReady(true);
      } catch (e) {
        console.error('WASM start failed:', e);
      }
    }

    start();

    // Polling
    intervalRef.current = window.setInterval(() => {
      if (!wasmModule || !wasmModule.poll_event) return;
      const json = wasmModule.poll_event();
      if (json) {
        const msg: Message = JSON.parse(json);
        setMessages((prev) => [...prev, msg]);
      }
    }, 100);

    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
      // Safe shutdown
      if (wasmModule && typeof wasmModule.shutdown === 'function') {
        wasmModule.shutdown();
      }
    };
  }, []);

  const handleSend = () => {
    if (input.trim() && wasmModule) {
      wasmModule.send_message('chat', input.trim());
      setMessages((prev) => [
        ...prev,
        { type: 'message', text: input, from: 'me' },
      ]);
      setInput('');
    }
  };

  return (
      <div className="min-h-screen bg-gray-900 text-white flex flex-col p-4 max-w-md mx-auto">
        <h1 className="text-2xl font-bold mb-4">P2P Chat</h1>
        <div className="flex-1 overflow-y-auto space-y-2 mb-4">
          {messages.map((msg, i) => (
              <div
                  key={i}
                  className={`p-2 rounded-lg ${
                      msg.type === 'connected' || msg.type === 'disconnected'
                          ? 'bg-green-800 text-center text-sm'
                          : msg.from === 'me'
                              ? 'bg-blue-600 self-end text-right'
                              : 'bg-gray-700'
                  }`}
              >
                {msg.type === 'connected' && (
                    <span>🟢 {msg.peer} connected</span>
                )}
                {msg.type === 'disconnected' && (
                    <span>🔴 {msg.peer} disconnected</span>
                )}
                {msg.type === 'message' && (
                    <>
                      <p className="text-xs opacity-70">{msg.from}</p>
                      <p>{msg.text}</p>
                    </>
                )}
              </div>
          ))}
        </div>
        <div className="flex gap-2">
          <input
              className="flex-1 p-2 rounded bg-gray-800 border border-gray-600 text-white"
              placeholder="Type a message..."
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSend()}
              disabled={!ready}
          />
          <button
              className="px-4 py-2 bg-blue-500 rounded disabled:opacity-50"
              onClick={handleSend}
              disabled={!ready}
          >
            Send
          </button>
        </div>
      </div>
  );
}

export default App;