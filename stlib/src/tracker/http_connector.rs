use reqwest::Client;

use crate::model::TrackerNetworkInfo;

use super::{TrackerConnector, TrackerEvent};

pub struct HttpTrackerConnector {
    client: Client,
}

impl TrackerConnector for HttpTrackerConnector {
    async fn announce(&mut self, request: &super::TrackerAnnounceRequest) -> Result<TrackerNetworkInfo, String> {
        let url = format!("{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}&event={}", 
            request.url,
            urlencoding::encode_binary(&request.info_hash),
            urlencoding::encode_binary(&request.peer_id),
            request.port,
            request.uploaded,
            request.downloaded,
            request.left,
            if request.compact { "1" } else { "0" },
            event_to_string(&request.event),
        );  
        
        let response = self.client
            .get(url)
            .send()
            .await
            .expect("Something went wrong with request")
            .bytes()
            .await 
            .expect("Couldn't convert response body to bytes");
        
        Ok(TrackerNetworkInfo::from_bencode(&response)?)
    }

}


fn event_to_string(event: &TrackerEvent) -> &'static str {
    match event {
        TrackerEvent::Completed => "completed",
        TrackerEvent::Started => "started",
        TrackerEvent::Stopped => "stopped",
        TrackerEvent::None => "",
    }
}

