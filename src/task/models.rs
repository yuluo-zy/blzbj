use crate::bilibili::models::{RoomInfo, UserInfo};

#[derive(Debug, Clone)]
pub enum RunningStatus {
    Stop,
    Wait,
    Record,
    Remix,
    Inject,
}

#[derive(Debug, Clone)]
pub enum StreamFormat {
    Flv,
    Ts,
    Fmp4,
}

#[derive(Debug, Clone)]
enum QualityNumber {
    K4 = 20000,
    Original = 10000,
    BluRayDolby = 401,
    BluRay = 400,
    UltraHD = 250,
    HD = 150,
    Smooth = 80,
}
#[derive(Debug, Clone)]
enum CoverSaveStrategy {
    DEFAULT,
    DEDUP
}

#[derive(Debug, Clone)]
pub struct TaskStatus {
    monitor_enabled: bool,
    recorder_enabled: bool,
    running_status: RunningStatus,
    stream_url: String,
    stream_host: String,
    dl_total: u64,
    dl_rate: u64,
    rec_elapsed: f64, // time elapsed
    rec_total: u64,
    rec_rate: u64,
    danmu_total: u64,
    danmu_rate: f64,
    real_stream_format: Option<StreamFormat>,
    real_quality_number: Option<QualityNumber>,
    recording_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TaskParam {
    // OutputSettings
    out_dir: String,
    path_template: String,
    filesize_limit: i32,
    duration_limit: i32,
    // BiliApiSettings
    base_api_urls: Vec<String>,
    base_live_api_urls: Vec<String>,
    base_play_info_api_urls: Vec<String>,
    // HeaderSettings
    user_agent: String,
    cookie: String,
    // DanmakuSettings
    danmu_uname: bool,
    record_gift_send: bool,
    record_free_gifts: bool,
    record_guard_buy: bool,
    record_super_chat: bool,
    save_raw_danmaku: bool,
    // RecorderSettings
    stream_format: StreamFormat,
    quality_number: QualityNumber,
    fmp4_stream_timeout: i32,
    read_timeout: i32,
    disconnection_timeout: Option<i32>,
    buffer_size: i32,
    save_cover: bool,
    cover_save_strategy: CoverSaveStrategy,
    // PostprocessingOptions
    remix_to_mp4: bool,
    inject_extra_metadata: bool,
}

pub struct TaskData {
    user_info: UserInfo,
    room_info: RoomInfo,
    task_status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoFileStatus {
    Recording,
    Remixing,
    Injecting,
    Completed,
    Missing,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DanmukuFileStatus {
    Recording,
    Completed,
    Missing,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct VideoFileDetail {
    pub path: String,
    pub size: i64,
    pub status: VideoFileStatus,
}

#[derive(Debug, Clone)]
pub struct DanmakuFileDetail {
    pub path: String,
    pub size: i64,
    pub status: DanmukuFileStatus,
}
