use std::{fs, io::{Read, Write}, net::{TcpListener, TcpStream}};

use format_bytes::{format_bytes, write_bytes};
use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

use crate::model::{PeerInfo, Torrent, TorrentInfo, TrackerNetworkInfo};

const LISTNER_PORT: u16 = 6881;

pub struct TorrentClient { 
    torrent: Torrent,
    peer_id: [u8; 20],
    client: Client,
}

impl TorrentClient {
    pub fn new(torrent: Torrent, peer_id: [u8; 20]) -> Self {
        Self { torrent, peer_id, client: Client::new() }
    }
}


impl TorrentClient {
    pub fn get_peers(&self, listener_port: u16) -> Result<TrackerNetworkInfo, reqwest::Error> {
        let file_length = match self.torrent.info {
            TorrentInfo::SingleFile { length , ..} => length,
            _ => 0,
        };

        let url = format!("{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}", 
            self.torrent.announce,
            urlencoding::encode_binary(&self.torrent.info.sha1_hash()),
            urlencoding::encode_binary(&self.peer_id).to_string(),
            listener_port,
            "0".to_string(),
            "0".to_string(),
            file_length.to_string(),
            1,
        );
        let response = self.client.get(url).send()?;
        let response = TrackerNetworkInfo::from_bencode(&response.bytes().unwrap()).unwrap();
        Ok(response)
    }
}

impl TorrentClient {
    pub fn handshake_peer(&self, peer_address: &str) -> Result<[u8; 20], std::io::Error> {
        const BUFFER_CAPACITY: usize = 1 + 19 + 8 + 20 + 20; 
        let mut buf = Vec::<u8>::with_capacity(BUFFER_CAPACITY);
        buf.write(&[19u8])?;
        buf.write(b"BitTorrent protocol")?;
        buf.write(&[0u8; 8])?;
        buf.write(&self.torrent.info.sha1_hash())?;
        buf.write(&self.peer_id)?;
        let mut socket: TcpStream = TcpStream::connect(peer_address)?;
        socket.write_all(&buf)?;
        buf.resize(BUFFER_CAPACITY, 0);
        socket.read(&mut buf)?;
        let info_hash: &[u8; 20] = &buf[1 + 19 + 8 .. 1 + 19 + 8 + 20].try_into().unwrap();
        if info_hash != &self.torrent.info.sha1_hash() {
            panic!("info_hash is not equal");
        }
        let mut peer_id = [0u8; 20];
        peer_id.copy_from_slice(&buf[1 + 19 + 8 + 20 .. 1 + 19 + 8 + 20 + 20]);
        Ok(peer_id)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, vec};

    use crate::{client::LISTNER_PORT, model::{PeerInfo, Torrent}};

    use super::TorrentClient;

    #[test]
    fn test_get_peers() {
        let content = fs::read("test-resources/sample.torrent").unwrap();
        let torrent: Torrent =  serde_bencode::from_bytes(&content).unwrap();
        let client = TorrentClient::new(torrent, b"00112233445566778899".to_owned());
        let tracker_info = client.get_peers(LISTNER_PORT).unwrap();
        assert_eq!(tracker_info.interval, 60);
        assert_eq!(tracker_info.peers, vec![
            PeerInfo {
                ip: String::from("165.232.33.77"),
                port: 51467,
            },
            PeerInfo {
                ip: String::from("178.62.85.20"),
                port: 51489,
            },
            PeerInfo {
                ip: String::from("178.62.82.89"),
                port: 51448,
            },
        ]);
    }

    #[test]
    fn test_handshake() {
        let content = fs::read("test-resources/sample.torrent").unwrap();
        let torrent: Torrent =  serde_bencode::from_bytes(&content).unwrap();
        let client = TorrentClient::new(torrent, b"00112233445566778899".to_owned());
       
        let peer_id: [u8; 20] = client.handshake_peer("165.232.33.77:51467").unwrap();
        println!("peer id = {}", hex::encode(peer_id));
    }
}