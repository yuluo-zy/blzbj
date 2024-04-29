#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub name: String,
    pub gender: String,
    pub face: String,
    pub uid: i32,
}

pub struct EnvSettings {
    settings_file: String,
    out_dir: String,
    log_dir: String
}