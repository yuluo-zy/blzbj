use std::str::FromStr;
use serde::{Deserialize, Serialize};
use utils::chrono::DateTime;
use utils::regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub uid: u64,
    pub room_id: u64,
    pub short_room_id: u64,
    pub area_id: u64,
    pub area_name: String,
    pub parent_area_id: u64,
    pub parent_area_name: String,
    pub live_status: u32,
    pub live_start_time: u64,
    pub online: u64,
    pub title: String,
    pub cover: String,
    pub tags: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub name: String,
    pub gender: String,
    pub face: String,
    pub uid: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LiveStatus {
    Preparing = 0,
    Live = 1,
    Round = 2,
}

impl RoomInfo {
    pub fn from_data(data: &serde_json::Value) -> Result<Self, String> {
        let live_start_time = if let Some(timestamp) = data.get("live_start_time").and_then(|v| v.as_i64()) {
            timestamp
        } else if let Some(time_string) = data.get("live_time").and_then(|v| v.as_str()) {
            if time_string == "0000-00-00 00:00:00" {
                0
            } else {
                let dt = DateTime::from_str(time_string).map_err(|e| e.to_string())?;
                dt.timestamp()
            }
        } else {
            return Err("Failed to init live_start_time".to_string());
        };

        let cover = data.get("cover").or(data.get("user_cover")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = if let Some(desc) = data.get("description").and_then(|v| v.as_str()) {
            let re = Regex::new(r"<br\s*/?>").unwrap();
            let cleaned_desc = re.replace_all(desc, "\n").to_string();
            cleaned_desc
        } else {
            "".to_string()
        };

        Ok(RoomInfo {
            uid: data.get("uid").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            room_id: data.get("room_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            short_room_id: data.get("short_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            area_id: data.get("area_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            area_name: data.get("area_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            parent_area_id: data.get("parent_area_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            parent_area_name: data.get("parent_area_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            live_status: match data.get("live_status").and_then(|v| v.as_i64()).unwrap_or(0) {
                0 => LiveStatus::Preparing,
                1 => LiveStatus::Live,
                2 => LiveStatus::Round,
                _ => LiveStatus::Preparing,
            },
            live_start_time,
            online: data.get("online").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            title: data.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            cover,
            tags: data.get("tags").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description,
        })
    }
}

impl UserInfo {
    pub fn from_web_api_data(data: &serde_json::Value) -> Result<Self, String> {
        Ok(UserInfo {
            name: data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            gender: data.get("sex").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            face: data.get("face").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            uid: data.get("mid").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        })
    }


    pub fn from_info_by_room(data: &serde_json::Value) -> Result<Self, String> {
        let room_info = data.get("room_info").ok_or("Missing room_info field")?;
        let anchor_info = data.get("anchor_info").ok_or("Missing anchor_info field")?;
        let base_info = anchor_info.get("base_info").ok_or("Missing base_info field")?;
        Ok(UserInfo {
            name: base_info.get("uname").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            gender: base_info.get("gender").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            face: base_info.get("face").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            uid: room_info.get("uid").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        })
    }
}
