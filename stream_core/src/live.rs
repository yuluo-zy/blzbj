use std::cmp::PartialEq;
use utils::async_trait::async_trait;
use utils::BResult;
use crate::live::LiveStatus::Live;

#[derive(Debug, Copy, Clone)]
pub enum StreamFormat {
    Flv,
    Fmp4,
}
#[derive(Debug, Copy, Clone)]
pub enum RecordingMode {
    Standard,
    Raw,
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LiveStatus {
    Live = 1,
    Offline = 2,
    Unknown = 3,
}
impl From<i32> for LiveStatus {
    fn from(value: i32) -> Self {
        match value {
            1 => LiveStatus::Live,
            2 => LiveStatus::Offline,
            3 => LiveStatus::Unknown,
            _ => LiveStatus::Unknown
        }
    }
}
#[derive(Debug, Copy, Clone)]
pub enum QualityNumber {
    P20000, // 4K
    P10000, // 原画
    P401, // 蓝光(杜比)
    P400, // 蓝光
    P250, // 超清
    P150, // 高清
    P80, // 流畅
}
impl From<i32> for QualityNumber {
    fn from(value: i32) -> Self {
        match value {
            20000 => QualityNumber::P20000,
            10000 => QualityNumber::P10000,
            401 => QualityNumber::P401,
            400 => QualityNumber::P400,
            250 => QualityNumber::P250,
            150 => QualityNumber::P150,
            80 => QualityNumber::P80,
            _ => QualityNumber::P250
        }
    }
}
impl From<QualityNumber> for i32 {
    fn from(value: QualityNumber) -> Self {
        match value {
              QualityNumber::P20000 => 20000,
            QualityNumber::P10000 => 10000,
            QualityNumber::P401 => 401,
            QualityNumber::P400 => 400,
            QualityNumber::P250 => 250,
            QualityNumber::P150 => 150,
            QualityNumber::P80 => 80,
        }
    }
}



#[derive(Debug, Clone)]
pub struct RoomInfo {
    uid: i32,
    room_id: i32,
    short_room_id: i32,
    area_id: i32,
    area_name: String,
    parent_area_id: i32,
    parent_area_name: String,
    live_status: LiveStatus,
    live_start_time: i64,
    online: i32,
    title: String,
    cover: String,
    tags: String,
    description: String,
}
impl RoomInfo {
    pub fn new(uid: i32,
               room_id: i32,
               short_room_id: i32,
               area_id: i32,
               area_name: String,
               parent_area_id: i32,
               parent_area_name: String,
               live_status: LiveStatus,
               live_start_time: i64,
               online: i32,
               title: String,
               cover: String,
               tags: String,
               description: String,) -> Self{
        Self {
            uid,
            room_id,
            short_room_id,
            area_id,
            area_name,
            parent_area_id,
            parent_area_name,
            live_status,
            live_start_time,
            online,
            title,
            cover,
            tags,
            description,
        }
    }

    pub fn is_living(&self) -> bool {
        self.live_status == Live
    }
}

#[async_trait]
pub trait  LiveTrait  {
    async fn room_info() -> BResult<RoomInfo>;

    fn stream_format() -> BResult<StreamFormat>;

    async fn is_living() -> BResult<bool>;

    async fn live_streams() -> BResult<Vec<String>>;
}

pub trait LiveMonitorTrait {}