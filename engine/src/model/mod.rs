use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct TorrentFile {
    pub announce: String,
    pub info: TorrentFileInfo,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TorrentFileInfo {
    MultiFile {
        name: String,
        #[serde(rename = "piece length")]
        piece_length: u64,
        pieces: Vec<u8>,
        files: Vec<FileEntry>,
    },
    SingleFile {
        name: String,
        #[serde(rename = "piece length")]
        piece_length: u64,
        pieces: Vec<u8>,
        length: u64,
    }
}

#[derive(Debug, Deserialize)]
pub struct FileEntry {
    path: Vec<String>,
    length: u64,
}

#[cfg(test)]
mod tests {
    
    #[test]
    fn test() {
        assert_eq!(1, 1);
    }


}