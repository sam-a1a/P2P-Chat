export type ChatEvent =
  | { type: 'message'; from: string; text: string }
  | { type: 'connected' | 'disconnected'; peer: string }
  | { type: 'published'; id: string; text: string }
  | { type: 'error'; message: string };
export type Status = 'disconnected' | 'connecting' | 'connected';
type Wasm = typeof import('../wasm/pkg/web_wasm');
let initialization: Promise<Wasm> | undefined;
function load(): Promise<Wasm> {
  return initialization ??= import('../wasm/pkg/web_wasm').then(async m => { await m.default(); return m; }).catch(e => { initialization = undefined; throw e; });
}
export class ChatClient {
  private wasm?: Wasm;
  private timer?: ReturnType<typeof setInterval>;
  private deadline?: ReturnType<typeof setTimeout>;
  private generation = 0;
  status: Status = 'disconnected';
  constructor(private event: (event: ChatEvent) => void, private changed: (status: Status) => void) {}
  private setStatus(status: Status) { this.status = status; this.changed(status); }
  async connect(address: string) {
    this.disconnect();
    const generation = ++this.generation;
    this.setStatus('connecting');
    try {
      const wasm = await load();
      if (generation !== this.generation) return;
      this.wasm = wasm;
      wasm.start_node(address.trim());
      this.deadline = setTimeout(() => { this.event({ type: 'error', message: 'Connection timed out. Check the address and that the desktop node is running.' }); this.disconnect(); }, 20000);
      this.timer = setInterval(() => {
        let json: string | undefined;
        while ((json = wasm.poll_event())) {
          const event = JSON.parse(json) as ChatEvent;
          if (event.type === 'connected') { clearTimeout(this.deadline); this.setStatus('connected'); }
          this.event(event);
          if (event.type === 'disconnected' || (event.type === 'error' && this.status === 'connecting')) { this.disconnect(); break; }
        }
      }, 50);
    } catch (e) {
      if (generation !== this.generation) return;
      this.event({ type: 'error', message: String(e) });
      this.disconnect();
    }
  }
  send(text: string) {
    if (this.status !== 'connected') throw new Error('Connect to a peer first.');
    this.wasm!.send_message(crypto.randomUUID(), text.trim());
  }
  disconnect() {
    ++this.generation;
    clearInterval(this.timer); clearTimeout(this.deadline);
    this.wasm?.shutdown(); this.wasm = undefined;
    this.setStatus('disconnected');
  }
}
export const defaultAddress = '/ip4/127.0.0.1/tcp/9090/ws';
export function savedAddress() { try { return localStorage.getItem('p2p-address') || defaultAddress; } catch { return defaultAddress; } }
export function saveAddress(address: string) { try { localStorage.setItem('p2p-address', address); } catch {} }
export function describe(event: ChatEvent): string {
  if (event.type === 'message') return `${event.from.slice(0, 12)}: ${event.text}`;
  if (event.type === 'published') return `You: ${event.text}`;
  if (event.type === 'error') return event.message;
  return `${event.peer.slice(0, 12)} ${event.type}`;
}
