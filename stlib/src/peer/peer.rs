use bit_vec::BitVec;

pub type PeerId = [u8; 20];

#[derive(Default)]
pub struct Peer {
    pub id: PeerId,
    pub ip: u32,
    pub port: u16,
    choked: bool,
    chocking: bool,
    interested: bool,
    bitfield: BitVec,
}

impl Peer {
    fn new(ip: u32, port: u16) -> Self {
        Self {
            ip,
            port,
            choked: true,
            chocking: true,
            ..Default::default()
        }
    }
}