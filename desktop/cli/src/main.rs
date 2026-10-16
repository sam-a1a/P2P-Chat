//! CLI test harness — lets you run the engine on your laptop to verify
//! LAN discovery and messaging before touching any mobile tooling.

use p2p::{
    identity::load_or_create_keypair,
    node::{NodeConfig, start_node_with_config},
    types::NodeEvent,
};
use std::io::BufRead;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut config = NodeConfig::default();
    let mut identity = "./identity.key".to_string();
    let mut dial = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--help" {
            println!(
                "p2p-chat-cli [--identity PATH] [--tcp MULTIADDR] [--websocket MULTIADDR] [--dial MULTIADDR]"
            );
            return Ok(());
        }
        let value = args
            .next()
            .ok_or_else(|| anyhow::anyhow!("Missing value for {arg}"))?;
        match arg.as_str() {
            "--identity" => identity = value,
            "--tcp" => config.tcp_address = value,
            "--websocket" => config.websocket_address = Some(value),
            "--dial" => dial = Some(value),
            _ => anyhow::bail!("Unknown argument: {arg}"),
        }
    }
    let keypair = load_or_create_keypair(&identity)?;
    log::info!("Local peer ID: {}", keypair.public().to_peer_id());

    let mut handle = start_node_with_config(keypair, config)?;
    if let Some(address) = dial {
        handle.dial(&address);
    }

    // Default topic for the CLI test.
    handle.subscribe("chat");

    // Event printer (background task)
    // We need to move event_rx into a task; shadow the handle fields.
    let (_command_tx, mut event_rx) = {
        // Destructure just what we need — _command_tx stays in scope below.
        use tokio::sync::mpsc::unbounded_channel;
        let evt_rx = std::mem::replace(
            // SAFETY: We're the only holder of event_rx at this point.
            &mut handle.event_rx,
            unbounded_channel::<NodeEvent>().1,
        );
        (handle.command_tx.clone(), evt_rx)
    };

    tokio::spawn(async move {
        while let Some(ev) = event_rx.recv().await {
            match &ev {
                NodeEvent::PeerDiscovered { peer } => println!("[+] Peer discovered: {peer}"),
                NodeEvent::PeerExpired { peer } => println!("[-] Peer expired: {peer}"),
                NodeEvent::ConnectionEstablished { peer, address } => {
                    println!("[✓] Connected: {peer} @ {address}")
                }
                NodeEvent::ConnectionClosed { peer } => println!("[✗] Disconnected: {peer}"),
                NodeEvent::PeerReady { peer } => println!("[ready] {peer}"),
                NodeEvent::MessagePublished { text } => println!("[sent] {text}"),
                NodeEvent::MessageReceived(msg) => {
                    let text = String::from_utf8_lossy(&msg.ciphertext);
                    println!("[msg] <{}> {}", &msg.from_peer[..8], text);
                }
                NodeEvent::ListeningOn { address } => println!("[*] Listening: {address}"),
                NodeEvent::Error { message } => println!("[!] Error: {message}"),
            }
        }
    });

    // Stdin → publish loop
    println!("Type a message and press Enter to broadcast on topic 'chat'.");
    println!("Type 'quit' to exit.\n");

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim() == "quit" {
            break;
        }
        if line.is_empty() {
            continue;
        }

        handle.publish("chat", line.into_bytes());
    }

    handle.shutdown();
    // Give the event loop a moment to flush.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    println!("Goodbye.");
    Ok(())
}
