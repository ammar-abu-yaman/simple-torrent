use bit_vec::BitVec;

pub enum PeerMessage {
    Choke, 
    Unchoke, 
    Interested,
    NotInterested,
    Have(u64),
    Bitfield(BitVec),
    Request {
        index: u64,
        begin: u64,
        length: u64,
    },
    Piece {
        index: u64,
        begin: u64,
        piece: Vec<u8>,
    },
    Cancel {
        index: u64,
        begin: u64,
        length: u64,
    },
}