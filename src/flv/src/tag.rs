
pub struct Tag {
    type_of: TagType,
    tag_flag: TagFlag,
    index: u64,
    size: u64,
    time_stamp: i32,
    data_hash: Option<String>,
    // ScriptTagBody
    extra_data: Option<TagExtraData>,
    // encode: Vec<>
}

pub enum TagFlag {
    None = 0,
    Header = 1 << 0,
    KeyFrame = 1 << 1,
    End = 1 << 2,
}

pub enum TagType {
    Unknown = 0,
    Audio = 8,
    Video = 9,
    Script = 18
}

pub struct TagExtraData {
    first_bytes: String,
    composition_time: i32,
    final_time: i32,
}
impl TagExtraData {
    pub fn should_serialize_composition(&self) -> bool {
        self.composition_time != i32::MIN
    }
    pub fn should_serialize_final_time(&self) -> bool {
        self.final_time != i32::MIN
    }
}

impl Tag {
    pub fn is_header(&self) -> bool {}
    pub fn is_end(&self) -> bool {}

    pub fn is_data(&self) -> bool {}

    pub fn is_non_key_frame_data(&self) -> bool {
    }

    pub fn is_keyframe_data(&self) -> bool {

    }

    pub async fn write_to(self) {}
}