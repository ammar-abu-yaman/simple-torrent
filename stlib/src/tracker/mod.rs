mod http_connector;
mod udp_connector;

pub use http_connector::*;
pub use udp_connector::*;

use crate::{model::{PeerInfo, Sha1Hash}, peer::peer::PeerId};

pub trait TrackerConnector {
    fn announce(request: &TrackerAnnounceRequest) -> Result<TrackerAnnounceResponse, String>;
    fn scrape(request: &TrackerScrapeRequest) -> Result<TrackerScrapeResponse, String> {
        panic!("Not implemented");
    }
}

pub struct TrackerAnnounceRequest {
    peer_id: PeerId,
    info_hash: Sha1Hash,
    downloaded: i32,
    left: u32,
    uploaded: u32,
    event: TrackerEvent,
    ip: u32,
    port: u16,
}
pub struct TrackerAnnounceResponse {
    interval: u32,
    seeders: u32,
    leechers: u32,
    peers: PeerInfo,
}

pub struct TrackerScrapeRequest {

}

pub struct TrackerScrapeResponse {

}

pub enum TrackerEvent {
    None = 0,
    Completed = 1,
    Started = 2,
    Stopped = 3,
}