use std::io;
use std::io::{Cursor, ErrorKind, Write};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use rand::random;
use crate::model::{PeerInfo, TrackerNetworkInfo};

use super::{TrackerAnnounceRequest, TrackerConnector};

const MINUTE_SECONDS: u64 = 60;
const CONNECT_REQUEST_PROTOCOL_ID: u64 = 0x41727101980;
const CONNECT_REQUEST_MIN_PACKET_SIZE: usize = 16;
const ANNOUNCE_REQUEST_MIN_PACKET_SIZE: usize = 20;
const IPV4_PORT_SIZE: usize = 4 + 2;


struct UdpTrackerConnector {
    key: u32,
    connection_id: Option<u64>,
    last_connect_timestamp: Instant,
}

impl UdpTrackerConnector {
    fn new() -> Self {
        Self {
            key: random(),
            connection_id: None,
            last_connect_timestamp: Instant::now(),
        }
    }
}

impl TrackerConnector for UdpTrackerConnector {
    async fn announce(&mut self, request: &super::TrackerAnnounceRequest) -> Result<TrackerNetworkInfo, String> {
        let mut socket = UdpSocket::bind("127.0.0.1:0").expect("Couldn't bind UDP socket");
        socket.connect(SocketAddr::new(IpAddr::from(request.ip.to_be_bytes()), request.port)).expect("Couldn't connect to tracker");
        let mut buf = [0u8; 2048];
        let mut timeout_multiplier = 0;
        loop {
            if self.connection_id.is_none() || self.last_connect_timestamp.elapsed().as_secs() >= MINUTE_SECONDS {
                match self.connect(&mut socket, &mut timeout_multiplier)
                    .await {
                    Ok(_) => {},
                    Err(err) => return Err(err.to_string()),
                }
            }
            if timeout_multiplier > 8 {
                return Err(String::from( "Announce request timed out"));
            }
            let transaction_id: u32 = random();
            let message = make_announce_request(self.connection_id.unwrap(), transaction_id, self.key, request);
            match send_recv_timeout(&mut socket, &mut buf, &message, timeout_multiplier) {
                Ok(n) if n >= ANNOUNCE_REQUEST_MIN_PACKET_SIZE => {
                    let mut cursor = Cursor::new(&mut buf[..]);
                    let action = cursor.read_u32::<BigEndian>().unwrap();
                    let tracker_transaction_id = cursor.read_u32::<BigEndian>().unwrap();
                    let interval = cursor.read_u32::<BigEndian>().unwrap();
                    let leechers = cursor.read_u32::<BigEndian>().unwrap();
                    let seeders = cursor.read_u32::<BigEndian>().unwrap();

                    if action != Action::Announce as u32 {
                        return Err(String::from("Response action isn't announce"));
                    }
                    if transaction_id != tracker_transaction_id {
                        return Err(String::from("Response transaction id doesn't equal generated transaction id"));
                    }

                    let peers_len = (n - ANNOUNCE_REQUEST_MIN_PACKET_SIZE) / IPV4_PORT_SIZE;
                    let peers: Vec<PeerInfo> = (0..peers_len)
                        .map(|_| {
                            let ip = cursor.read_u32::<BigEndian>().unwrap();
                            let port = cursor.read_u16::<BigEndian>().unwrap();
                            SocketAddr::new(IpAddr::from(ip.to_be_bytes()), port)
                        })
                        .map(|socket_addr| PeerInfo { socket_addr })
                        .collect();

                    return Ok(TrackerNetworkInfo {
                        interval,
                        leechers: Some(leechers),
                        seeders: Some(seeders),
                        peers,
                    });
                },
                Err(err) if err.kind() == ErrorKind::TimedOut => { timeout_multiplier += 1; }
                Ok(_) => return Err(String::from("Announce response too short")),
                Err(err) => return Err(err.to_string()),
            }
        }

    }
}


impl UdpTrackerConnector {
    
    async fn connect(&mut self, socket: &mut UdpSocket, timeout_multiplier: &mut u32) -> Result<(), io::Error> {
        let transaction_id: u32 = random();
        let message = make_connection_request(transaction_id);

        let mut buf = [0u8; 256];
        loop {
            if *timeout_multiplier > 8 {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "Connect request timed out"));
            }
            match send_recv_timeout(socket, &mut buf, &message, *timeout_multiplier) {
                Ok(n) if n >= CONNECT_REQUEST_MIN_PACKET_SIZE => {
                    let mut cursor = Cursor::new(&mut buf[..]);
                    let action = cursor.read_u32::<BigEndian>().unwrap();
                    let tracker_transaction_id = cursor.read_u32::<BigEndian>().unwrap();
                    let connection_id = cursor.read_u64::<BigEndian>().unwrap();
                    if action != Action::Connect as u32 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Response action isn't connect"));
                    }
                    if transaction_id != tracker_transaction_id {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Response transaction id doesn't equal generated transaction id"));
                    }
                    self.connection_id = Some(connection_id);
                    self.last_connect_timestamp = Instant::now();
                },
                Err(err) if err.kind() == ErrorKind::TimedOut => { *timeout_multiplier += 1; }
                Ok(_) => return Err(io::Error::new(io::ErrorKind::InvalidData, "Connect response too short")),
                Err(err) => return Err(err),
            }
        }
    }
}

enum Action {
    Connect = 0,
    Announce = 1,
}

fn send_recv_timeout(socket: &mut UdpSocket, buf: &mut [u8], message: &[u8], timeout_multiplier: u32) -> Result<usize, io::Error> {
    let read_timeout = Duration::from_secs(15u64 * 2u64.pow(timeout_multiplier));
    if socket.send(message)? != message.len() {
        return Err(io::Error::new(io::ErrorKind::Other, "Couldn't write full message to socket"));
    }
    socket.set_read_timeout(Some(read_timeout)).unwrap();
    socket.recv(buf)
}


fn make_connection_request(translation_id: u32) -> [u8; 16] {
    use byteorder::BigEndian;

    let mut buf = [0u8; 16];
    let mut cursor =  Cursor::new(&mut buf[..]);
    cursor.write_u64::<BigEndian>(CONNECT_REQUEST_PROTOCOL_ID).unwrap();
    cursor.write_u32::<BigEndian>(Action::Connect as u32).unwrap();
    cursor.write_u32::<BigEndian>(translation_id).unwrap();
    buf
}

fn make_announce_request(connection_id: u64, transaction_id: u32, key: u32, request: &TrackerAnnounceRequest) -> [u8; 98] {
    use byteorder::BigEndian;

    let mut buf = [0u8; 98];
    let mut cursor = Cursor::new(&mut buf[..]);
    cursor.write_u64::<BigEndian>(connection_id).unwrap();
    cursor.write_u32::<BigEndian>(Action::Announce as u32).unwrap();
    cursor.write_u32::<BigEndian>(transaction_id).unwrap();
    cursor.write_all(&request.info_hash).unwrap();
    cursor.write_all(&request.peer_id).unwrap();
    cursor.write_u64::<BigEndian>(request.downloaded as u64).unwrap();
    cursor.write_u64::<BigEndian>(request.left as u64).unwrap();
    cursor.write_u64::<BigEndian>(request.uploaded as u64).unwrap();
    cursor.write_u32::<BigEndian>(request.event as u32).unwrap();
    cursor.write_u32::<BigEndian>(0).unwrap();
    cursor.write_u32::<BigEndian>(key).unwrap();
    cursor.write_i32::<BigEndian>(-1).unwrap(); // num_want
    cursor.write_u16::<BigEndian>(request.port).unwrap();

    buf
}
