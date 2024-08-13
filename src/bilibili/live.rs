use serde::Deserialize;
use utils::{reqwest, TError};
use utils::error::LiveError;
use utils::reqwest::Client;
use crate::bilibili::api::{BaseApi, WebApi};
use crate::bilibili::models::{RoomInfo, UserInfo};


#[derive(Debug, Deserialize)]
struct ResponseData {}

#[derive(Debug, Deserialize)]
struct LiveStatus(i32);

struct Live {
    room_id: i32,
    room_info: Option<RoomInfo>,
    user_info: Option<UserInfo>,
    no_flv_stream: bool,
    webapi: WebApi,
}

impl Live {
    pub fn new(room_id: i32, user_agent: String, cookie: String) -> Self {
        let client = Client::builder().build().unwrap();
        let headers = Self::update_headers(room_id, &user_agent, &cookie);
        Self {
            room_id,
            room_info: None,
            user_info: None,
            no_flv_stream: false,
            webapi: WebApi::new(client, headers, Some(room_id)),
        }
    }

    fn update_headers(room_id: i32, user_agent: &str, cookie: &str) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Referer", format!("https://live.bilibili.com/{}", room_id).parse().unwrap());
        headers.insert("User-Agent", user_agent.parse().unwrap());
        headers.insert("Cookie", cookie.parse().unwrap());
        headers
    }

    async fn init(&mut self) -> Result<(), LiveError> {
        self.room_info = Some(self.get_room_info().await?);
        self.user_info = Some(self.get_user_info(&self.room_info.unwrap().uid).await?);

        if self.is_living() {
            let streams = self.get_live_streams(None).await?;
            if !streams.is_empty() {
                self.no_flv_stream = !streams.iter().any(|stream| stream.format == "flv");
            }
        }

        Ok(())
    }


    async fn deinit(&self) {
        // Do any cleanup if necessary
    }

    fn is_living(&self) -> bool {
        matches!(self.room_info, Some(RoomInfo { live_status: 1, .. }))
    }

    async fn get_live_status(&self) -> Result<LiveStatus, LiveError> {
        // Implement the logic to get live status
        Ok(LiveStatus(1))
    }

    async fn get_room_info(&self) -> Result<RoomInfo, LiveError> {
        let res = self.webapi.get_info_by_room(self.room_id).await?;
        Ok(res)
    }

    async fn get_user_info(&self, uid: u64) -> Result<UserInfo, LiveError> {
        // Implement the logic to get user info
        Ok(UserInfo {})
    }

    async fn get_live_streams(&self, qn: Option<i32>) -> Result<Vec<Stream>, LiveError> {
        // Implement the logic to get live streams
        Ok(vec![])
    }
}

#[derive(Debug, Deserialize)]
struct Stream {
    format: String,
    // Define other fields based on your requirements
}
