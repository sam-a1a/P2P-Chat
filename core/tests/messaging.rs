use libp2p::identity::Keypair;
use p2p::{
    node::{NodeConfig, NodeHandle, start_node_with_config},
    types::NodeEvent,
};
use tokio::time::{Duration, timeout};

fn node() -> NodeHandle {
    start_node_with_config(
        Keypair::generate_ed25519(),
        NodeConfig {
            tcp_address: "/ip4/127.0.0.1/tcp/0".into(),
            websocket_address: Some("/ip4/127.0.0.1/tcp/0/ws".into()),
        },
    )
    .unwrap()
}
async fn event(node: &mut NodeHandle, matches: impl Fn(&NodeEvent) -> bool) -> NodeEvent {
    timeout(Duration::from_secs(15), async {
        loop {
            let event = node.event_rx.recv().await.expect("node stopped");
            if matches(&event) {
                return event;
            }
        }
    })
    .await
    .expect("timed out waiting for node event")
}
async fn exchange(websocket: bool) {
    let mut a = node();
    let mut b = node();
    a.subscribe("chat");
    b.subscribe("chat");
    let listen = event(&mut a, |e| matches!(e, NodeEvent::ListeningOn { address } if address.ends_with("/ws") == websocket)).await;
    if let NodeEvent::ListeningOn { address } = listen {
        b.dial(&address);
    }
    event(&mut a, |e| matches!(e, NodeEvent::PeerReady { .. })).await;
    event(&mut b, |e| matches!(e, NodeEvent::PeerReady { .. })).await;
    a.publish("chat", "Hello مرحبا".as_bytes().to_vec());
    let received = event(&mut b, |e| matches!(e, NodeEvent::MessageReceived(_))).await;
    if let NodeEvent::MessageReceived(message) = received {
        assert_eq!(message.ciphertext, "Hello مرحبا".as_bytes());
    }
    b.publish("chat", b"reply".to_vec());
    let received = event(&mut a, |e| matches!(e, NodeEvent::MessageReceived(_))).await;
    if let NodeEvent::MessageReceived(message) = received {
        assert_eq!(message.ciphertext, b"reply");
    }
    a.shutdown();
    b.shutdown();
}
#[tokio::test]
async fn tcp_bidirectional_messages() {
    exchange(false).await;
}
#[tokio::test]
async fn websocket_bidirectional_messages() {
    exchange(true).await;
}
#[tokio::test]
async fn invalid_dial_and_publish_without_peers_are_reported() {
    let mut node = node();
    node.dial("not-an-address");
    event(
        &mut node,
        |e| matches!(e, NodeEvent::Error { message } if message.contains("invalid address")),
    )
    .await;
    node.subscribe("chat");
    node.publish("chat", b"hello".to_vec());
    event(
        &mut node,
        |e| matches!(e, NodeEvent::Error { message } if message.contains("publish")),
    )
    .await;
    node.shutdown();
}
