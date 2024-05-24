use serde::Deserialize;

#[derive(Debug)]
pub struct TrackerNetworkInfo {
    pub interval: u64,
    pub peers: Vec<PeerInfo>,
} 

#[derive(Debug, PartialEq)]
pub struct PeerInfo {
    pub ip: String,
    pub port: u16,
}

impl PeerInfo {
    pub fn to_socket_addrs(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

impl TrackerNetworkInfo {
    pub fn from_bencode(bytes: &[u8]) -> Result<Self, String> {
        let response: TrackerDiscoveryResponse =  serde_bencode::from_bytes(bytes).unwrap();
        match response {
            TrackerDiscoveryResponse::Error { failure_reason } => Err(failure_reason),
            TrackerDiscoveryResponse::Response { interval, peers } => Ok(Self {
                interval,
                peers: Self::parse_peers(peers),
            })
        }
    }

    fn parse_peers(peers: TrackersPeersResponse) -> Vec<PeerInfo> {
        match peers {
            TrackersPeersResponse::Legacy(peers) => peers.into_iter().map(|p| PeerInfo { ip: p.ip, port: p.port}).collect(),
            TrackersPeersResponse::Compact(peers) => peers.chunks(6).map(|chunk| PeerInfo {
                ip: format!("{}.{}.{}.{}", chunk[0], chunk[1], chunk[2], chunk[3]),
                port: u16::from_be_bytes(chunk[4..].try_into().unwrap()),
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
        interval: u64,
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
    pub id: [u8; 20],
    pub ip: String,
    pub port: u16
}