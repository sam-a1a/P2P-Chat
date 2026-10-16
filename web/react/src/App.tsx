import { useEffect, useRef, useState } from 'react';
import { ChatClient, describe, saveAddress, savedAddress, type ChatEvent, type Status } from '../../shared/client';
import '../../shared/chat.css';
export default function App() {
  const [address, setAddress] = useState(savedAddress);
  const [status, setStatus] = useState<Status>('disconnected');
  const [events, setEvents] = useState<ChatEvent[]>([]);
  const [text, setText] = useState('');
  const [pending, setPending] = useState(false);
  const client = useRef<ChatClient | null>(null);
  const log = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const node = new ChatClient(e => {
      setEvents(old => [...old.slice(-499), e]);
      if (e.type === 'published') { setPending(false); setText(current => current.trim() === e.text ? '' : current); }
      if (e.type === 'error') setPending(false);
    }, status => { setStatus(status); if (status === 'disconnected') setPending(false); });
    client.current = node;
    return () => { node.disconnect(); client.current = null; };
  }, []);
  useEffect(() => { log.current?.scrollTo(0, log.current.scrollHeight); }, [events]);
  function send(e: React.FormEvent) {
    e.preventDefault();
    try { setPending(true); client.current?.send(text); }
    catch (error) { setPending(false); setEvents(old => [...old, { type: 'error', message: String(error) }]); }
  }
  return <main>
    <h1>P2P Chat · React</h1><p>Connect to a desktop peer and chat on the shared LAN channel.</p>
    <label htmlFor="address">Peer address</label>
    <form className="row" onSubmit={e => { e.preventDefault(); saveAddress(address); void client.current?.connect(address); }}>
      <input id="address" value={address} onChange={e => setAddress(e.target.value)} disabled={status !== 'disconnected'} required />
      <button disabled={status !== 'disconnected'}>Connect</button>
      <button type="button" onClick={() => client.current?.disconnect()} disabled={status === 'disconnected'}>Disconnect</button>
    </form>
    <div className="status" role="status">Status: {status}</div>
    <div className="log" role="log" aria-label="Messages" ref={log}>
      {!events.length && <p>No messages yet.</p>}
      {events.map((event, i) => <div key={i} className={`entry ${event.type === 'error' ? 'error' : ''}`}>{describe(event)}</div>)}
    </div>
    <form className="row" onSubmit={send}>
      <input aria-label="Message" placeholder="Type a message…" value={text} onChange={e => setText(e.target.value)} disabled={status !== 'connected'} maxLength={16000} />
      <button disabled={status !== 'connected' || pending || !text.trim()}>Send</button>
    </form>
    <small>“You” appears after the local peer accepts publication; it is not a delivery receipt. Messages are visible to peers on this channel and are not saved after reload.</small>
  </main>;
}
