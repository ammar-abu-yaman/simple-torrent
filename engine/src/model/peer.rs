use bit_vec::BitVec;

pub const CHOKE: u8 = 0;
pub const UNCHOKE: u8 = 1;
pub const INTERESTED: u8 = 2;
pub const NOT_INTERESTED: u8 = 3;
pub const HAVE: u8 = 4;
pub const BITFIELD: u8 = 5;
pub const REQUEST: u8 = 6;
pub const PIECE: u8 = 7;
pub const CANCEL: u8 = 8;

#[derive(Debug)]
pub enum PeerMessage {
    Choke, 
    Unchoke, 
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(BitVec),
    Request {
        index: u32,
        begin: u32,
        length: u32,
    },
    Piece {
        index: u32,
        begin: u32,
        piece: Vec<u8>,
    },
    Cancel {
        index: u32,
        begin: u32,
        length: u32,
    },
    KeepAlive,
}