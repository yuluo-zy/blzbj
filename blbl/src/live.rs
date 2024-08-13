use std::collections::HashMap;
use core::live::{LiveTrait, RoomInfo, LiveStatus};
use utils::BResult;
use crate::api::{AppClient};

pub struct Live {
    room_id: usize,
    user_agent: Option<String>,
    cookie: Option<String>,
    client: AppClient,
    room_info: Option<RoomInfo>,
    no_flv_stream: bool,
}

impl Default for Live {
    fn default() -> Self {
        Self {
            room_id: 0,
            user_agent: None,
            cookie: None,
            client: AppClient::new(None),
            room_info: None,
            no_flv_stream: false,
        }
    }
}
impl Live {
    pub async fn init(mut self, room_id: usize) -> BResult<Self> {
        self.room_id = room_id;

        Ok(self)
    }

    pub fn update_user_info(&mut self, user_agent: &str, cookie: &str) {
        self.user_agent = Some(user_agent.to_string());
        self.cookie = Some(cookie.to_string());
        let mut heads = HashMap::new();
        heads.insert("Referer".to_string(), format!("https://live.bilibili.com/{}", self.room_id));
        heads.insert("User-Agent".to_string(), user_agent.to_string());
        heads.insert("Cookie".to_string(), cookie.to_string());
        self.client.update_heads(heads);
    }

    async fn room_info(&mut self) -> BResult<()> {
        let response = self.client.get_info_by_room(self.room_id).await?;
        let room_info = parse_room_info(response)?;
        self.room_info = Some(room_info);
        Ok(())
    }

    // async fn live_status(&self) -> BResult<LiveStatus> {
    //     self.room_info().await?;
    //     Ok()
    // }
}

fn parse_room_info(response: serde_json::Value) -> BResult<RoomInfo> {

    let room_info = RoomInfo::new(

    )
    Ok()
}

impl LiveTrait for Live {
    fn room_info() -> BResult<core::live::RoomInfo> {
        todo!()
    }

    fn stream_format() -> BResult<core::live::StreamFormat> {
        todo!()
    }

    fn is_living() -> BResult<bool> {
        todo!()
    }
}