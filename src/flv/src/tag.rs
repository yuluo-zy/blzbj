use blake3::Hasher;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use crate::amf::ScriptTagBody;
use crate::h264_nalu::H264Nalu;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    type_of: TagType,
    flag: TagFlag,
    index: u64,
    size: u64,
    time_stamp: i32,
    data_hash: Option<String>,

    #[serde(skip)]
    binary_data: Option<Bytes>,
    script_tag_body: Option<ScriptTagBody>,
    extra_data: Option<TagExtraData>,
    nalus: Vec<H264Nalu>,
    #[serde(rename = "BinaryData", serialize_with = "option_hex_serialize", deserialize_with = "option_hex_deserialize", skip_serializing_if = "Option::is_none")]
    binary_data_for_serialization: Option<Vec<u8>>,
}

fn option_hex_serialize<S>(bytes: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
{
    if let Some(ref bytes) = *bytes {
        serializer.serialize_some(&*vec_to_hex_string(bytes))
    } else {
        serializer.serialize_none()
    }
}

// 反序列化十六进制字符串为 Vec<u8>
fn option_hex_deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: serde::Deserializer<'de>,
{
    let hex_str: Option<String> = Option::deserialize(deserializer)?;
    Ok(hex_str.map(|s| hex_string_to_vec(&s)))
}

// 实现二进制数据和十六进制字符串之间的转换
fn hex_string_to_vec(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
        .collect()
}

// 将 Vec<u8> 转换为十六进制字符串
fn vec_to_hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
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
    Script = 18,
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
    fn should_serialize_binary_data_for_serialization_use_only(&self) -> bool {
        self.flag.contains(TagFlag::Header)
    }

    #[inline]
    fn should_serialize_script_data(&self) -> bool {
        self.flag == TagType::Script
    }

    #[inline]
    fn should_serialize_nalus(&self) -> bool {
        self.flag == TagType::Video && !self.flag.contains(TagFlag::Header)
    }

    // pub fn close(&self) {}
    //
    // pub fn is_header(&self) -> bool {}
    // pub fn is_end(&self) -> bool {}
    //
    // pub fn is_data(&self) -> bool {}
    // 
    // pub fn is_non_key_frame_data(&self) -> bool {}
    //
    // pub fn is_keyframe_data(&self) -> bool {}

    pub fn binary_data_for_serialization_use_only(&self) {}

    pub fn update_data_hash(&mut self) {
        match &self.binary_data {
            None => { self.data_hash = None }
            Some(value) => {
                let hash = Hasher::new().update(&value).finalize();
                self.data_hash = Some(hash.to_hex().to_string());

                for nalu in self.nalus.iter_mut() {
                    let bytes_left = value.len() - nalu.start_position;
                    let hash_size = if bytes_left >= nalu.full_size as usize {
                        nalu.full_size
                    } else {
                        bytes_left as u32
                    };
                    let hash = Hasher::new().update(&value[nalu.start_position..nalu.start_position + hash_size]).finalize();
                    nalu.nalu_hash = Some(hash.to_hex().to_string());
                    if bytes_left < nalu.full_size as usize {
                        nalu.nalu_hash = Some(nalu.clone().nalu_hash.unwrap() + "-PARTIAL");
                    }
                }
            }
        }
    }

    pub async fn write_to(self) {}
}