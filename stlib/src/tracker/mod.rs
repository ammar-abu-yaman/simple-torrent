mod http_connector;
mod udp_connector;

pub use http_connector::*;
pub use udp_connector::*;

use crate::{model::{PeerInfo, Sha1Hash, TrackerNetworkInfo}, peer::peer::PeerId};

pub trait TrackerConnector {
    async fn announce(&mut self, request: &TrackerAnnounceRequest) -> Result<TrackerNetworkInfo, String>;
    async fn scrape(&mut self, request: &TrackerScrapeRequest) -> Result<TrackerScrapeResponse, String> {
        unimplemented!("Not implemented");
    }
}

pub struct TrackerAnnounceRequest {
    url: String,
    peer_id: PeerId,
    info_hash: Sha1Hash,
    downloaded: i32,
    left: u32,
    uploaded: u32,
    event: TrackerEvent,
    ip: u32,
    port: u16,
    compact: bool,
}

pub struct TrackerScrapeRequest {

}

pub struct TrackerScrapeResponse {

}

#[derive(Copy, Clone)]
pub enum TrackerEvent {
    None = 0,
    Completed = 1,
    Started = 2,
    Stopped = 3,
}