use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use serde::Deserialize;
use crate::model::Sha1Hash;

#[derive(Debug)]
pub struct TrackerNetworkInfo {
    pub interval: u32,
    pub leechers: Option<u32>,
    pub seeders: Option<u32>,
    pub peers: Vec<PeerInfo>,
} 

#[derive(Debug, PartialEq)]
pub struct PeerInfo {
    pub socket_addr: SocketAddr,
}

impl TrackerNetworkInfo {
    pub fn from_bencode(bytes: &[u8]) -> Result<Self, String> {
        let response: TrackerDiscoveryResponse =  serde_bencode::from_bytes(bytes).unwrap();
        match response {
            TrackerDiscoveryResponse::Error { failure_reason } => Err(failure_reason),
            TrackerDiscoveryResponse::Response { interval, leechers, seeders, peers } => Ok(Self {
                interval,
                leechers,
                seeders,
                peers: Self::parse_peers(peers),
            })
        }
    }

    fn parse_peers(peers: TrackersPeersResponse) -> Vec<PeerInfo> {
        match peers {
            TrackersPeersResponse::Legacy(peers) => peers.into_iter()
                .map(|peer| PeerInfo { socket_addr: SocketAddr::new(peer.ip.parse().unwrap(), peer.port) })
                .collect(),
            TrackersPeersResponse::Compact(peers) => peers.chunks(6).map(|chunk| PeerInfo {
                socket_addr: SocketAddr::new(IpAddr::from(
                    Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3])), u16::from_be_bytes(chunk[4..].try_into().unwrap())
                ),
            }).collect()
        }
    }
}



#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TrackerDiscoveryResponse {
    Error {
        #[serde(rename = "failure reason")]
        failure_reason: String,
    },
    Response {
        interval: u32,
        leechers: Option<u32>,
        seeders: Option<u32>,
        peers: TrackersPeersResponse,
    }
} 


#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TrackersPeersResponse {
    #[serde(with = "serde_bytes")]
    Compact(Vec<u8>),
    Legacy(Vec<LegacyPeerInfo>),
}

#[derive(Debug, Deserialize)]
struct LegacyPeerInfo {
    #[serde(with = "serde_bytes")]
    pub id: Sha1Hash,
    pub ip: String,
    pub port: u16
}