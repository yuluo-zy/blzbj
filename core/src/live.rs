use utils::BResult;

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
#[derive(Debug, Copy, Clone)]
pub enum LiveStatus {
    Live,
    Offline,
    Unknown,
    // Add other variants as needed
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
}
pub trait  LiveTrait  {
    fn room_info() -> BResult<RoomInfo>;

    fn stream_format() -> BResult<StreamFormat>;

    fn is_living() -> BResult<bool>;
}

pub trait LiveMonitor {}