use serde::{Deserialize, Serialize};
use crate::model::Sha1Hash;

use crate::util::common::sha1_hash;

#[derive(Debug, Clone, Deserialize)]
pub struct Torrent {
    pub announce: String,
    #[serde(rename = "announce-list", default = "empty_vec")]
    pub announce_list: Vec<Vec<String>>,
    #[serde(rename = "created by", default = "empty_string")]
    pub created_by: String,
    #[serde(default = "empty_string")]
    pub comment: String,

    #[serde(default = "utf_8")]
    pub encoding: String,

    pub info: TorrentInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TorrentInfo {
    MultiFile {
        name: String,
        #[serde(rename = "piece length")]
        piece_length: u64,
        #[serde(with = "serde_bytes")]
        pieces: Vec<u8>,
        files: Vec<FileEntry>,
    },
    SingleFile {
        name: String,
        #[serde(rename = "piece length")]
        piece_length: u64,
        #[serde(with = "serde_bytes")]
        pieces: Vec<u8>,
        length: u64,
    }
}

impl TorrentInfo {
    pub fn sha1_hash(&self) -> Sha1Hash {
        let serialized_bytes = serde_bencode::to_bytes(self)
            .expect("Couldn't serialize info dictionary for torrent file");
        sha1_hash(serialized_bytes)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: Vec<String>,
    pub length: u64,
}

fn empty_vec() -> Vec<Vec<String>> {
    vec![]
}

fn empty_string() -> String {
    String::new()
}

fn utf_8() -> String {
    String::from("UTF-8")
}

#[cfg(test)]
mod tests { 
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_single_correctly() {
        let content = fs::read("test-resources/torrent/sample.torrent").unwrap();
        let torrent: Torrent =  serde_bencode::from_bytes(&content).unwrap();

        assert_eq!(torrent.announce, "http://bittorrent-test-tracker.codecrafters.io/announce");
        assert_eq!(torrent.encoding, "UTF-8");
        assert!(matches!(torrent.info, TorrentInfo::SingleFile { .. }));
    }

    #[test]
    fn test_info_hash() {
        let content = fs::read("test-resources/torrent/sample.torrent").unwrap();
        let torrent: Torrent =  serde_bencode::from_bytes(&content).unwrap();
        let hash = hex::encode(torrent.info.sha1_hash());
        assert_eq!(hash, "d69f91e6b2ae4c542468d1073a71d4ea13879a7f")
    }

}
