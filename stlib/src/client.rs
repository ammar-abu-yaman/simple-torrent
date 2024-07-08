// use std::io::{self, Read, Write};
//
// use bit_vec::BitVec;
// use reqwest::blocking::Client;
//
// use crate::model::{PeerMessage, Sha1Hash, Torrent, TrackerNetworkInfo};
// use crate::model::{CHOKE, UNCHOKE, INTERESTED, NOT_INTERESTED, HAVE, BITFIELD, REQUEST, PIECE, CANCEL};
//
// pub const LISTNER_PORT: u16 = 6881;
// pub const PIECE_BLOCK_SIZE: u32 = 2u32.pow(14);
//
// #[derive(Debug, Clone)]
// pub struct TorrentClient {
//     torrent: Torrent,
//     peer_id: Sha1Hash,
//     client: Client,
// }
//
// impl TorrentClient {
//     pub fn new(torrent: Torrent, peer_id: Sha1Hash) -> Self {
//         Self { torrent, peer_id, client: Client::new() }
//     }
// }
//
//
// impl TorrentClient {
//     pub fn get_peers(&self, listener_port: u16) -> Result<TrackerNetworkInfo, reqwest::Error> {
//         let file_length = match self.torrent.info {
//             TorrentInfo::SingleFile { length , ..} => length,
//             _ => 0,
//         };
//
//         let url = format!("{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",
//             self.torrent.announce,
//             urlencoding::encode_binary(&self.torrent.info.sha1_hash()),
//             urlencoding::encode_binary(&self.peer_id),
//             listener_port,
//             "0",
//             "0",
//             file_length,
//             1,
//         );
//         let response = self.client.get(url).send()?;
//         let response = TrackerNetworkInfo::from_bencode(&response.bytes().unwrap()).unwrap();
//         Ok(response)
//     }
// }
//
// impl TorrentClient {
//
//     pub fn handshake_peer<S>(&self, endpoint: &mut S) -> Result<Sha1Hash, std::io::Error>
//     where S: Read + Write
//     {
//         const BUFFER_CAPACITY: usize = 1 + 19 + 8 + 20 + 20;
//         let mut buf = Vec::<u8>::with_capacity(BUFFER_CAPACITY);
//         buf.write_all(&[19u8])?;
//         buf.write_all(b"BitTorrent protocol")?;
//         buf.write_all(&[0u8; 8])?;
//         buf.write_all(&self.torrent.info.sha1_hash())?;
//         buf.write_all(&self.peer_id)?;
//         endpoint.write_all(&buf)?;
//         endpoint.flush()?;
//         buf.resize(BUFFER_CAPACITY, 0);
//         endpoint.read_exact(&mut buf)?;
//         let info_hash: &Sha1Hash = &buf[1 + 19 + 8 .. 1 + 19 + 8 + 20].try_into().unwrap();
//         if info_hash != &self.torrent.info.sha1_hash() {
//             panic!("info_hash is not equal");
//         }
//         let mut peer_id = [0u8; 20];
//         peer_id.copy_from_slice(&buf[1 + 19 + 8 + 20 .. 1 + 19 + 8 + 20 + 20]);
//         Ok(peer_id)
//     }
// }
//
// impl TorrentClient {
//     pub fn read_message(&self, endpoint: &mut impl Read) -> Result<PeerMessage, io::Error> {
//         let mut buf = [0u8; 4];
//         endpoint.read_exact(&mut buf)?;
//
//         let length = u32::from_be_bytes(buf) as usize;
//         if length == 0 {
//             return Ok(PeerMessage::KeepAlive);
//         }
//         let mut buf = vec![0u8; length];
//         endpoint.read_exact(&mut buf)?;
//         let message_type = buf[0];
//         let buf = &buf[1..];
//         match message_type {
//             CHOKE => Ok(PeerMessage::Choke),
//             UNCHOKE => Ok(PeerMessage::Unchoke),
//             INTERESTED => Ok(PeerMessage::Interested),
//             NOT_INTERESTED => Ok(PeerMessage::NotInterested),
//             HAVE if length == 5 => Ok(PeerMessage::Have(u32::from_be_bytes(buf.try_into().unwrap()))),
//             BITFIELD => Ok(PeerMessage::Bitfield(BitVec::from_bytes(&buf))),
//             REQUEST | CANCEL if length == 1 + 4 * 3 => {
//                 let index = u32::from_be_bytes(buf[.. 4].try_into().unwrap());
//                 let begin = u32::from_be_bytes(buf[4 .. 4 * 2].try_into().unwrap());
//                 let length = u32::from_be_bytes(buf[4 * 2 .. 4 * 3].try_into().unwrap());
//                 if message_type == REQUEST {
//                     Ok(PeerMessage::Request { index, begin, length })
//                 } else {
//                     Ok(PeerMessage::Cancel { index, begin, length })
//                 }
//             }
//             PIECE if length >= 1 + 4 * 2 =>  {
//                 let index = u32::from_be_bytes(buf[.. 4].try_into().unwrap());
//                 let begin = u32::from_be_bytes(buf[4 .. 4 * 2].try_into().unwrap());
//                 let piece = buf[4 * 2 ..].to_vec();
//                 Ok(PeerMessage::Piece { index, begin, piece })
//             }
//             _ => {
//                 Err(io::Error::from(io::ErrorKind::InvalidData))
//             },
//         }
//     }
//     pub fn write_message(&self, endpoint: &mut impl Write, message: &PeerMessage)  -> Result<(), std::io::Error> {
//         match message {
//             PeerMessage::Choke => {
//                 endpoint.write_all(&1u32.to_be_bytes())?;
//                 endpoint.write_all(&[CHOKE])?;
//             },
//             PeerMessage::Unchoke => {
//                 endpoint.write_all(&1u32.to_be_bytes())?;
//                 endpoint.write_all(&[UNCHOKE])?;
//             },
//             PeerMessage::Interested => {
//                 endpoint.write_all(&1u32.to_be_bytes())?;
//                 endpoint.write_all(&[INTERESTED])?;
//             },
//             PeerMessage::NotInterested => {
//                 endpoint.write_all(&1u32.to_be_bytes())?;
//                 endpoint.write_all(&[NOT_INTERESTED])?;
//             },
//             PeerMessage::Have(index) => {
//                 endpoint.write_all(&5u32.to_be_bytes())?;
//                 endpoint.write_all(&[HAVE])?;
//                 endpoint.write_all(&index.to_be_bytes())?;
//             },
//             PeerMessage::Bitfield(bits) => {
//                 let payload = bits.to_bytes();
//                 endpoint.write_all(&((1 + payload.len()) as u32).to_be_bytes())?;
//                 endpoint.write_all(&[BITFIELD])?;
//                 endpoint.write_all(&payload)?;
//             },
//             PeerMessage::Request { index, begin, length }  | PeerMessage::Cancel { index, begin, length } => {
//                 let message_type = if let PeerMessage::Request { .. } = message { [REQUEST] } else { [CANCEL] };
//                 endpoint.write_all( &13u32.to_be_bytes())?;
//                 endpoint.write_all(&message_type)?;
//                 endpoint.write_all(&[index.to_be_bytes(), begin.to_be_bytes(), length.to_be_bytes()].concat())?;
//             },
//             PeerMessage::Piece { index, begin, piece } => {
//                 endpoint.write_all(&((9 + piece.len()) as u32).to_be_bytes())?;
//                 endpoint.write_all(&[PIECE])?;
//                 endpoint.write_all(&[index.to_be_bytes(), begin.to_be_bytes()].concat())?;
//                 endpoint.write_all(&piece)?;
//             },
//             PeerMessage::KeepAlive => {
//                 endpoint.write_all(&0u32.to_be_bytes())?;
//             },
//         };
//         endpoint.flush()?;
//         Ok(())
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use std::{fs, io, net::TcpStream, time::Duration};
//
//     use crate::{
//         client::{LISTNER_PORT, PIECE_BLOCK_SIZE},
//         model::*,
//         util::common::sha1_hash
//     };
//
//     use super::TorrentClient;
//
//     #[test]
//     fn test_get_peers() {
//         let content = fs::read("test-resources/sample.torrent").unwrap();
//         let torrent: Torrent =  serde_bencode::from_bytes(&content).unwrap();
//         let client = TorrentClient::new(torrent, b"00112233445566778899".to_owned());
//         let tracker_info = client.get_peers(LISTNER_PORT).unwrap();
//         assert_eq!(tracker_info.interval, 60);
//         assert_eq!(tracker_info.peers, vec![
//             PeerInfo {
//                 ip: String::from("165.232.33.77"),
//                 port: 51467,
//             },
//             PeerInfo {
//                 ip: String::from("178.62.85.20"),
//                 port: 51489,
//             },
//             PeerInfo {
//                 ip: String::from("178.62.82.89"),
//                 port: 51448,
//             },
//         ]);
//     }
//
//     #[test]
//     fn test_handshake() {
//         let content = fs::read("test-resources/sample.torrent").unwrap();
//         let torrent: Torrent =  serde_bencode::from_bytes(&content).unwrap();
//         let client = TorrentClient::new(torrent, b"00112233445566778899".to_owned());
//         let mut socket = TcpStream::connect("165.232.33.77:51467").unwrap();
//         let peer_id: Sha1Hash = client.handshake_peer(&mut socket).unwrap();;
//     }
//
//     #[test]
//     fn test_download_file() {
//         let content = fs::read("test-resources/sample.torrent").unwrap();
//         let torrent: Torrent =  serde_bencode::from_bytes(&content).unwrap();
//         let client = TorrentClient::new(torrent.clone(), b"00112233445566778899".to_owned());
//         let tracker_info = client.get_peers(LISTNER_PORT).unwrap();
//         let peer = &tracker_info.peers[1];
//         let mut socket = TcpStream::connect(peer.to_socket_addrs()).unwrap();
//         socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
//         socket.set_write_timeout(Some(Duration::from_secs(2))).unwrap();
//         client.handshake_peer(&mut socket).unwrap();
//         let bitfield_message = client.read_message(&mut socket).unwrap();
//         assert!(matches!(bitfield_message, PeerMessage::Bitfield(_)));
//         client.write_message(&mut socket, &PeerMessage::Interested).unwrap();
//         let unchoke_message = client.read_message(&mut socket).unwrap();
//         assert!(matches!(unchoke_message, PeerMessage::Unchoke));
//         let mut pieces_data = vec![];
//         if let TorrentInfo::SingleFile { name, piece_length, pieces, length } = &torrent.info {
//             for (piece_index, piece_hash) in match &torrent.info {
//                 TorrentInfo::MultiFile { pieces, .. } | TorrentInfo::SingleFile { pieces, .. } => pieces.chunks(20).enumerate(),
//             } {
//                 loop {
//                     let mut piece_data = vec![];
//                     let truncated_piece_length = {
//                         let start_index = piece_length * piece_index as u64;
//                         piece_length.clone().min(length - start_index) as u32
//                     };
//                     for offset in (0..truncated_piece_length as u32).step_by(PIECE_BLOCK_SIZE as usize) {
//                         let truncated_block_length = PIECE_BLOCK_SIZE.min(truncated_piece_length - offset);
//                          client.write_message(&mut socket, &PeerMessage::Request {
//                             index: piece_index as u32,
//                             begin: offset,
//                             length: truncated_block_length
//                         }).unwrap();
//
//                         let piece_message = read_non_keepalive_non_error_message(&client, &mut socket);
//                         if let Ok(PeerMessage::Piece { index, begin, piece }) = piece_message {
//                             assert_eq!((index, begin), (piece_index as u32, offset));
//                             assert_eq!(piece.len(), truncated_block_length as usize);
//                             piece_data.extend(piece);
//                         } else {
//                             panic!("Recieved a message that is not a piece: {piece_message:#?}");
//                         }
//                     }
//                     if sha1_hash(&piece_data) != piece_hash {
//                         continue;
//                     }
//                     pieces_data.push(piece_data);
//                     break;
//                 }
//             }
//         }
//         if let TorrentInfo::SingleFile { name, .. } = &torrent.info {
//             let piece_data = pieces_data.concat();
//             fs::write(name, piece_data).unwrap();
//         }
//     }
//
//     fn read_non_keepalive_non_error_message(client: &TorrentClient, socket: &mut TcpStream) -> Result<PeerMessage, std::io::Error> {
//         loop {
//             let message = client.read_message(socket);
//             match &message {
//                 Err(err) if err.kind() == io::ErrorKind::TimedOut => panic!("Read timed out"),
//                 Ok(PeerMessage::Piece { .. }) => return Ok(message.unwrap()),
//                 _ => continue,
//             }
//         }
//     }
// }