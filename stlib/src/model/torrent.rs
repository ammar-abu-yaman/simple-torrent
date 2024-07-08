use std::fs::File;
use std::{fs, io};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::model::{SHA1_HASH_LEN, Sha1Hash};

use crate::util::common::sha1_hash;

#[derive(Debug)]
pub struct Torrent {
    pub announce: String,
    pub announce_list: Vec<String>,
    pub created_by :String,
    pub comment: String,
    pub encoding: String,
    pub root_name: String,
    pub piece_length: u64,
    pub pieces: Vec<Sha1Hash>,
    pub info_hash: Sha1Hash,
    pub variant: TorrentVariant,
}

#[derive(Debug)]
pub enum TorrentVariant {
    SingleFile(u64),
    MultiFile(Vec<FileEntry>)
}

#[derive(Debug, PartialEq, Eq)]
pub struct FileEntry {
    pub path: PathBuf,
    pub length: u64,
}

impl Torrent {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        Self::from_bencode(&fs::read(path)?)
    }

    pub fn from_bencode(content: &[u8]) -> Result<Self, io::Error> {
        let TorrentBencode {
            announce,
            announce_list,
            created_by,
            comment,
            encoding,
            info,
        } = match serde_bencode::from_bytes(content) {
            Ok(torrent) => torrent,
            Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidInput, err)),
        };
        let info_hash = info.sha1_hash();
        let (name, piece_length, pieces, variant) = match info {
            TorrentInfo::MultiFile { name, piece_length, pieces, files } => {
                (name, piece_length, pieces, TorrentVariant::MultiFile(files.into_iter()
                    .map(|file| {
                        let FileEntryBencode { length, path } = file;
                        let mut path_buf = PathBuf::new();
                        for component in path {
                            path_buf.push(component);
                        }
                        FileEntry { path: path_buf, length: file.length }
                    }).collect()))
            }
            TorrentInfo::SingleFile { name, piece_length, pieces, length }
                => (name, piece_length, pieces, TorrentVariant::SingleFile(length))
        };
        if pieces.len() % SHA1_HASH_LEN != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, String::from("Pieces hashes aren't multiple of 20")));
        }
        Ok(Self {
            announce,
            announce_list: announce_list.into_iter().flatten().collect(),
            created_by,
            comment,
            encoding,
            root_name: name,
            piece_length,
            pieces: pieces.chunks(SHA1_HASH_LEN).map(|hash| hash.try_into().unwrap()).collect(),
            info_hash,
            variant,
        })
    }
}

impl Torrent {
    pub fn is_single_file(&self) -> bool {
        match self.variant {
            TorrentVariant::SingleFile(..) => true,
            TorrentVariant::MultiFile(..) => false,
        }
    }

    pub fn is_multi_file(&self) -> bool {
        match self.variant {
            TorrentVariant::SingleFile(..) => false,
            TorrentVariant::MultiFile(..) => true,
        }
    }

    pub fn file_len(&self) -> Option<u64> {
        if let TorrentVariant::SingleFile(len) = self.variant {
            Some(len)
        } else {
            None
        }
    }

    pub fn get_files(&self) -> Option<&[FileEntry]> {
        if let TorrentVariant::MultiFile(entries) = &self.variant {
            Some(entries)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct TorrentBencode {
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
enum TorrentInfo {
    MultiFile {
        name: String,
        #[serde(rename = "piece length")]
        piece_length: u64,
        #[serde(with = "serde_bytes")]
        pieces: Vec<u8>,
        files: Vec<FileEntryBencode>,
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
struct FileEntryBencode {
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

    #[test]
    fn test_parse_single_file_torrent() {
        let torrent = Torrent::from_file("test-resources/torrent/sample.torrent").unwrap();
        assert_eq!(torrent.announce, "http://bittorrent-test-tracker.codecrafters.io/announce");
        assert_eq!(torrent.encoding, "UTF-8");
        assert_eq!(torrent.created_by, "mktorrent 1.1");
        assert_eq!(torrent.piece_length, 32768);
        assert_eq!(torrent.comment, "");
        assert_eq!(torrent.root_name, "sample.txt");
        assert_eq!(hex::encode(torrent.info_hash), "d69f91e6b2ae4c542468d1073a71d4ea13879a7f");
        assert!(matches!(torrent.variant, TorrentVariant::SingleFile(92063)));
    }

    #[test]
    fn test_parse_multi_torrent() {
        let torrent = Torrent::from_file("test-resources/torrent/bunny.torrent").unwrap();
        assert_eq!(torrent.announce, "udp://tracker.leechers-paradise.org:6969");
        assert_eq!(torrent.announce_list, vec![
            "udp://tracker.leechers-paradise.org:6969".to_string(),
            "udp://tracker.coppersurfer.tk:6969".to_string(),
            "udp://tracker.opentrackr.org:1337".to_string(),
            "udp://explodie.org:6969".to_string(),
            "udp://tracker.empire-js.us:1337".to_string(),
            "wss://tracker.btorrent.xyz".to_string(),
            "wss://tracker.openwebtorrent.com".to_string(),
            "wss://tracker.fastcast.nz".to_string(),
        ]);
        assert_eq!(torrent.comment, "WebTorrent <https://webtorrent.io>");
        assert_eq!(torrent.created_by, "WebTorrent <https://webtorrent.io>");
        assert_eq!(torrent.encoding, "UTF-8");
        assert!(torrent.is_multi_file());
        let files = vec![
            FileEntry { path: PathBuf::from("Big Buck Bunny.en.srt"), length: 140 },
            FileEntry { path: PathBuf::from("Big Buck Bunny.mp4"), length: 276134947 },
            FileEntry { path: PathBuf::from("poster.jpg"), length: 310380 },
        ];
        assert_eq!(torrent.get_files(), Some(&files[..]));
        assert_eq!(torrent.root_name, "Big Buck Bunny");
        assert_eq!(torrent.piece_length, 262144);
        
    }

}
