use std::collections::HashMap;
use stream_core::live::{LiveTrait, RoomInfo, LiveStatus, QualityNumber};
use crate::api::{WebClient};
use anyhow::{anyhow, Result};

pub struct Live {
    room_id: usize,
    user_agent: Option<String>,
    cookie: Option<String>,
    client: WebClient,
    room_info: Option<RoomInfo>,
    no_flv_stream: bool,
}

impl Default for Live {
    fn default() -> Self {
        Self {
            room_id: 0,
            user_agent: None,
            cookie: None,
            client: WebClient::new(None),
            room_info: None,
            no_flv_stream: false,
        }
    }
}
impl Live {
    pub async fn init(mut self, room_id: usize) -> Result<Self> {
        self.room_id = room_id;
        self.room_info().await?;
        self.no_flv_stream = true;
        if self.is_living(){
            self.
        }
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

    async fn room_info(&mut self) -> Result<()> {
        let response = self.client.get_info_by_room(self.room_id).await?;
        let room_info = parse_room_info(response)?;
        self.room_info = Some(room_info);
        Ok(())
    }

    fn is_living(&self) -> bool {
      match self.room_info {
          None => false,
          Some(ref room_info) => room_info.is_living()
      }
    }

    async fn get_live_streams(&self, qn: QualityNumber) -> Result<()>{
        let play_infos = self.client.get_room_play_infos(self.room_id, qn.into()).await?;

        Ok(())
    }
}

fn parse_room_info(response: serde_json::Value) -> Result<RoomInfo> {
    let data = &response["data"]["room_info"];
    if data.is_null() {
        return Err(anyhow!("Missing room_info field"));
    }
    let room_info = RoomInfo::new(
        data["uid"].as_i64().unwrap_or_default() as i32,
        data["room_id"].as_i64().unwrap_or_default() as i32,
        data["short_id"].as_i64().unwrap_or_default() as i32,
        data["area_id"].as_i64().unwrap_or_default() as i32,
        data["area_name"].as_str().unwrap_or_default().to_string(),
        data["parent_area_id"].as_i64().unwrap_or_default() as i32,
        data["parent_area_name"].as_str().unwrap_or_default().to_string(),
        LiveStatus::from(data["live_status"].as_i64().unwrap_or_default() as i32),
        data["live_start_time"].as_i64().unwrap_or_default(),
        data["online"].as_i64().unwrap_or_default() as i32,
        data["title"].as_str().unwrap_or_default().to_string(),
        data["cover"].as_str().unwrap_or_default().to_string(),
        data["tags"].as_str().unwrap_or_default().to_string(),
        data["description"].as_str().unwrap_or_default().to_string(),
    );
    Ok(room_info)
}

// impl LiveTrait for Live {
//     fn room_info() -> Result<core::live::RoomInfo> {
//         todo!()
//     }
//
//     fn stream_format() -> Result<core::live::StreamFormat> {
//         todo!()
//     }
//
//     fn is_living() -> Result<bool> {
//         todo!()
//     }
// }