//! Integration tests for networking and peer discovery logic.

use std::net::IpAddr;
use std::str::FromStr;
use p2p_walkie_talkie::walkie_talkie::WalkieTalkie;
use p2p_walkie_talkie::peer::PeerInfo;

#[tokio::test]
async fn test_peer_info_creation() {
    let peer = PeerInfo {
        id: "test-id".to_string(),
        name: "TestPeer".to_string(),
        ip: IpAddr::from_str("127.0.0.1").unwrap(),
        port: 8080,
    };
    assert_eq!(peer.name, "TestPeer");
    assert_eq!(peer.port, 8080);
}

#[tokio::test]
async fn test_walkie_talkie_new() {
    let wt = WalkieTalkie::new("Tester".to_string(), 9000);
    assert_eq!(wt.name, "Tester");
    assert_eq!(wt.port, 9000);
}

// More tests for networking and peer discovery can be added here, using mocks or test doubles.
