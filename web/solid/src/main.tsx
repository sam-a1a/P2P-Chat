import { createSignal, onCleanup } from 'solid-js';
import { render } from '@solidjs/web';
import { ChatClient, describe, saveAddress, savedAddress, type ChatEvent, type Status } from '../../shared/client';
import '../../shared/chat.css';
function App() {
  const [address, setAddress] = createSignal(savedAddress());
  const [status, setStatus] = createSignal<Status>('disconnected');
  const [events, setEvents] = createSignal<ChatEvent[]>([]);
  const [text, setText] = createSignal('');
  let log: HTMLDivElement | undefined;
  const client = new ChatClient(event => {
    setEvents(old => [...old.slice(-499), event]);
    requestAnimationFrame(() => log?.scrollTo(0, log.scrollHeight));
  }, setStatus);
  onCleanup(() => client.disconnect());
  function send(event: SubmitEvent) {
    event.preventDefault();
    try { client.send(text()); setText(''); }
    catch (error) { setEvents(old => [...old, { type: 'error', message: String(error) }]); }
  }
  return <main>
    <h1>P2P Chat · Solid 2.0</h1><p>Connect to a desktop peer and chat on the shared LAN channel.</p>
    <label for="address">Peer address</label>
    <form class="row" onSubmit={e => { e.preventDefault(); saveAddress(address()); void client.connect(address()); }}>
      <input id="address" value={address()} onInput={e => setAddress(e.currentTarget.value)} disabled={status() !== 'disconnected'} required />
      <button disabled={status() !== 'disconnected'}>Connect</button>
      <button type="button" onClick={() => client.disconnect()} disabled={status() === 'disconnected'}>Disconnect</button>
    </form>
    <div class="status" role="status">Status: {status()}</div>
    <div class="log" role="log" aria-label="Messages" ref={log}>
      {events().length === 0 && <p>No messages yet.</p>}
      {events().map(event => <div class={`entry ${event.type === 'error' ? 'error' : ''}`}>{describe(event)}</div>)}
    </div>
    <form class="row" onSubmit={send}>
      <input aria-label="Message" placeholder="Type a message…" value={text()} onInput={e => setText(e.currentTarget.value)} disabled={status() !== 'connected'} maxlength={16000} />
      <button disabled={status() !== 'connected' || !text().trim()}>Send</button>
    </form>
    <small>“You” appears after the local peer accepts publication; it is not a delivery receipt. Messages are visible to peers on this channel and are not saved after reload.</small>
  </main>;
}
render(() => <App />, document.getElementById('root')!);
