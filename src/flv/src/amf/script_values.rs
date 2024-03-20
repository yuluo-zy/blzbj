use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::amf::{ScriptDataType, ScriptDataValue, ScriptDataValueTrait};
use anyhow::Result;
use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use chrono::{DateTime, TimeZone, Utc, FixedOffset, Offset};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ScriptDataBoolean {
    pub value: bool,
}

impl ScriptDataBoolean {
    pub fn new(value: bool) -> Self {
        ScriptDataBoolean {
            value,
        }
    }
}

impl ScriptDataValueTrait for ScriptDataBoolean {
    fn data_type(&self) -> ScriptDataType {
        ScriptDataType::Boolean
    }

    // 异步写入 ScriptDataBoolean 到流
    async fn write_to<W>(self, writer: &mut W) -> Result<()>
        where
            W: AsyncWrite + Unpin,
    {
        writer.write_u8(ScriptDataType::Boolean as u8).await?;
        writer.write_u8(self.value as u8).await?;
        Ok(())
    }
}

// 比较和转换的实现
impl From<bool> for ScriptDataBoolean {
    fn from(value: bool) -> Self {
        ScriptDataBoolean::new(value)
    }
}

impl From<ScriptDataBoolean> for bool {
    fn from(data: ScriptDataBoolean) -> Self {
        data.value
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ScriptDataDate {
    value: DateTime<Utc>,
}

impl ScriptDataDate {
    pub fn new(value: DateTime<Utc>) -> Self {
        ScriptDataDate { value }
    }

    pub fn from_unix_time(date_time: i64, local_date_time_offset: i32) -> Self {
        let fixed_offset = FixedOffset::east(local_date_time_offset * 60); // 转换为秒
        let datetime_with_offset = fixed_offset.timestamp_millis(date_time);
        ScriptDataDate { value: datetime_with_offset.into() }
    }
}

impl ScriptDataValueTrait for ScriptDataDate {
    fn data_type(&self) -> ScriptDataType {
        ScriptDataType::Date
    }

    async fn write_to<W>(self, writer: &mut W) -> Result<()> where W: AsyncWrite + Unpin + Send {
        let date_time = self.value.timestamp_millis() as f64;
        let local_date_time_offset = self.value.offset().fix().local_minus_utc() as i16;
        writer.write_u8(ScriptDataType::Date as u8)?;
        writer.write_f64(date_time)?;
        writer.write_i16(local_date_time_offset)?;
        Ok(())
    }
}

// 实现类型转换功能
impl From<DateTime<Utc>> for ScriptDataDate {
    fn from(dt: DateTime<Utc>) -> Self {
        ScriptDataDate::new(dt)
    }
}

impl From<ScriptDataDate> for DateTime<Utc> {
    fn from(sdd: ScriptDataDate) -> Self {
        sdd.value
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ScriptDataEcmaArray {
    pub value: HashMap<String, ScriptDataValue>,
}

impl ScriptDataEcmaArray {
    pub fn new() -> Self {
        ScriptDataEcmaArray {
            value: HashMap::new(),
        }
    }
}

impl ScriptDataValueTrait for ScriptDataEcmaArray {
    fn data_type(&self) -> ScriptDataType {
        ScriptDataType::EcmaArray
    }

    async fn write_to<W>(self, writer: &mut W) -> Result<()> where W: AsyncWrite + Unpin + Send {
        writer.write_u8(ScriptDataType::EcmaArray as u8).await?;
        writer.write_u32(self.value.len() as u32).await?;

        for (key, value) in self.value {
            let key_bytes = key.as_bytes();
            if key_bytes.len() > u16::MAX as usize {
                return Err(anyhow::anyhow!("Key length exceeds u16::MAX"));
            }

            writer.write_u16(key_bytes.len() as u16).await?;
            writer.write_all(key_bytes).await?;
            value.write_to(writer).await?;
        }

        // AMF0 object end marker
        writer.write_all(&[0, 0, 9]).await?;
        Ok(())
    }
}

impl From<HashMap<String, ScriptDataValue>> for ScriptDataEcmaArray {
    fn from(value: HashMap<String, ScriptDataValue>) -> Self {
        ScriptDataEcmaArray { value }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ScriptDataLongString {
    pub value: String,
}

impl ScriptDataLongString {
    pub fn new(value: String) -> Self {
        ScriptDataLongString { value }
    }
}

impl ScriptDataValueTrait for ScriptDataLongString {
    fn data_type(&self) -> ScriptDataType {
        ScriptDataType::LongString
    }

    async fn write_to<W>(self, writer: &mut W) -> Result<()> where W: AsyncWrite + Unpin + Send {
        let bytes = self.value.as_bytes();
        writer.write_u8(ScriptDataType::LongString as u8).await?;
        writer.write_u32(bytes.len() as u32).await?;
        writer.write_all(bytes).await?;
        Ok(())
    }
}

impl From<String> for ScriptDataLongString {
    fn from(value: String) -> Self {
        ScriptDataLongString::new(value)
    }
}

impl From<ScriptDataLongString> for String {
    fn from(script_data: ScriptDataLongString) -> Self {
        script_data.value
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptDataNull;

impl ScriptDataValueTrait for ScriptDataNull {
    fn data_type(&self) -> ScriptDataType {
        ScriptDataType::Null
    }

    async fn write_to<W>(self, writer: &mut W) -> Result<()> where W: AsyncWrite + Unpin + Send {
        writer.write_u8(ScriptDataType::Null as u8).await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ScriptDataNumber {
    pub value: f64,
}

impl ScriptDataNumber {
    pub fn new(value: f64) -> Self {
        ScriptDataNumber { value }
    }
}

impl ScriptDataValueTrait for ScriptDataNumber {
    fn data_type(&self) -> ScriptDataType {
        ScriptDataType::Number
    }

    async fn write_to<W>(self, writer: &mut W) -> Result<()> where W: AsyncWrite + Unpin + Send {
        writer.write_u8(ScriptDataType::Number as u8).await?;
        writer.write_f64(self.value).await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ScriptDataObject {
    value: HashMap<String, ScriptDataValue>,
}

impl ScriptDataObject {
    pub fn new() -> Self {
        ScriptDataObject {
            value: HashMap::new(),
        }
    }
}

impl ScriptDataValueTrait for ScriptDataObject {
    fn data_type(&self) -> ScriptDataType {
        ScriptDataType::Object
    }

    async fn write_to<W>(self, writer: &mut W) -> Result<()> where W: AsyncWrite + Unpin + Send {
        writer.write_u8(ScriptDataType::Object as u8)?;

        for (key, value) in self.value {
            let key_bytes = key.as_bytes();
            if key_bytes.len() > u16::MAX as usize {
                return Err(anyhow::anyhow!("Key length exceeds u16::MAX"));
            }

            writer.write_u16(key_bytes.len() as u16)?;
            writer.write_all(key_bytes)?;
            value.write_to(writer)?;
        }

        // AMF0 object end marker
        writer.write_all(&[0, 0, 9])?;
        Ok(())
    }
}

impl From<HashMap<String, ScriptDataValue>> for ScriptDataObject {
    fn from(value: HashMap<String, ScriptDataValue>) -> Self {
        ScriptDataObject { value }
    }
}

impl From<ScriptDataObject> for HashMap<String, ScriptDataValue> {
    fn from(script_data: ScriptDataObject) -> Self {
        script_data.value
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptDataReference {
    pub value: u16,
}

impl ScriptDataReference {
    pub fn new(value: u16) -> Self {
        ScriptDataReference { value }
    }
}

impl ScriptDataValueTrait for ScriptDataReference {
    fn data_type(&self) -> ScriptDataType {
        ScriptDataType::Reference
    }

    async fn write_to<W>(self, writer: &mut W) -> Result<()> where W: AsyncWrite + Unpin + Send {
        writer.write_u8(ScriptDataType::Reference as u8)?;
        writer.write_u16(self.value)?;
        Ok(())
    }
}

impl From<u16> for ScriptDataReference {
    fn from(value: u16) -> Self {
        ScriptDataReference::new(value)
    }
}

impl From<ScriptDataReference> for u16 {
    fn from(reference: ScriptDataReference) -> Self {
        reference.value
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ScriptDataStrictArray {
    pub value: Vec<ScriptDataValue>,
}

impl ScriptDataStrictArray {
    pub fn new() -> Self {
        ScriptDataStrictArray { value: Vec::new() }
    }
    pub fn push(&mut self, value: ScriptDataValue) {
        self.value.push(value);
    }
}

impl ScriptDataValueTrait for ScriptDataStrictArray {
    fn data_type(&self) -> ScriptDataType {
        ScriptDataType::StrictArray
    }

    async fn write_to<W>(self, writer: &mut W) -> Result<()> where W: AsyncWrite + Unpin + Send {
        writer.write_u8(ScriptDataType::StrictArray as u8)?;
        writer.write_u32(self.value.len() as u32)?;

        for item in self.value {
            item.write_to(writer)?;
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ScriptDataString {
    pub value: String,
}

impl ScriptDataString {
    pub fn new(value: String) -> Self {
        ScriptDataString { value }
    }
}

impl ScriptDataValueTrait for ScriptDataString {
    fn data_type(&self) -> ScriptDataType {
        ScriptDataType::String
    }

    async fn write_to<W>(self, writer: &mut W) -> Result<()> where W: AsyncWrite + Unpin + Send {
        let bytes = self.value.as_bytes();
        if bytes.len() > u16::MAX as usize {
            return Err(anyhow::anyhow!("Key length exceeds u16::MAX"));
        }

        writer.write_u8(self.get_type() as u8)?;
        writer.write_u16(bytes.len() as u16)?;
        writer.write_all(bytes)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ScriptDataUndefined;

impl ScriptDataValueTrait for ScriptDataUndefined {
    fn data_type(&self) -> ScriptDataType {
        ScriptDataType::Undefined
    }

    async fn write_to<W>(self, writer: &mut W) -> Result<()> where W: AsyncWrite + Unpin + Send {
        writer.write_u8(self.get_type() as u8)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ScriptTagBody {
    values: Vec<ScriptDataValue>,
}

impl ScriptTagBody {
    pub fn new(values: Vec<ScriptDataValue>) -> Self {
        ScriptTagBody { values }
    }
    pub fn parse_json(json: &str) -> serde_json::Result<Self> {
        let values: Vec<ScriptDataValue> = serde_json::from_str(json)?;
        Ok(ScriptTagBody::new(values))
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self.values)
    }

    pub async fn parse<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let mut list: Vec<ScriptDataValue> = Vec::new();

        while let Ok(item) = Self::parse_value(reader).await {
            list.push(item);
        }
        Ok(ScriptTagBody { values: list })
    }

    pub async fn parse_value<R: AsyncRead + Unpin>(reader: &mut R) -> Result<ScriptDataValue> {
        let type_byte = reader.read_u8().await?;
        let type_byte = num_enum::FromPrimitive::from_primitive(type_byte);
        match type_byte {
            ScriptDataType::Number => {
                let number = reader.read_f64().await?;
                return Ok(ScriptDataValue::Number(ScriptDataNumber::new(number)));
            }
            ScriptDataType::Boolean => {
                let boolean = reader.read_u8().await? != 0;
                return Ok(ScriptDataValue::Boolean(ScriptDataBoolean::new(boolean)));
            }
            ScriptDataType::String => {
                let str_ = read_script_data_string(reader, false).await?;
                return Ok(ScriptDataValue::String(ScriptDataString::new(str_)));
            }
            // ScriptDataType::Object => {}
            // todo: 复杂嵌套类型实现
            ScriptDataType::MovieClip => {
                return Err(anyhow::anyhow!("MovieClip is not supported"));
            }
            ScriptDataType::Null => {
                return Ok(ScriptDataValue::Undefined(ScriptDataUndefined));
            }
            ScriptDataType::Undefined => {
                return Ok(ScriptDataValue::Undefined(ScriptDataUndefined));
            }
            ScriptDataType::Reference => {
                let ref_ = reader.read_u16().await?;
                return Ok(ScriptDataValue::Reference(ScriptDataReference::new(ref_)));
            }
            ScriptDataType::ObjectEndMarker => {
                return Err(anyhow::anyhow!("Read ObjectEndMarker"));
            }
            ScriptDataType::StrictArray => {
                let length = reader.read_u32().await?;
                let mut array = ScriptDataStrictArray::new();
                for _i in length {
                    array.push(Self::parse_value(reader).await?);
                }
                return Ok(ScriptDataValue::StrictArray(array))
            }
            ScriptDataType::Date => {
                let date_time = reader.read_f64().await?;
                let offset_ = reader.read_i16().await?;
                let seconds = (date_time as i64) / 1000;
                // 然后取得毫秒中的剩余部分
                let nanoseconds = ((date_time % 1000.0) * 1_000_000.0) as u32;

                // 使用 chrono 创建 DateTime<Utc>
                let date_time = Utc.timestamp_opt(seconds, nanoseconds)
                    .single()
                    .ok_or_else(|| Err(anyhow::anyhow!("Invalid timestamp for DateTime<Utc>")))?;

                return Ok(ScriptDataValue::Date(ScriptDataDate::new(date_time)))
            }
            ScriptDataType::LongString => {
                let str_ = read_long_string(reader).await?;
                return Ok(ScriptDataValue::LongString(ScriptDataLongString::new(str_)));
            }
            _ => return Err(anyhow::anyhow!("Unknown ScriptDataValueType")),
        }
    }

    // pub fn to_bytes<W>(self, writer: &mut W) -> Result<Vec<u8>>  where W: AsyncWrite + Unpin + Send {
    // let mut buf = BytesMut::new();
    //     self.write_to(&mut buf)?;
    //     Ok(bytes)
    // }

    async fn write_to<W>(self, writer: &mut W) -> Result<()> where W: AsyncWrite + Unpin + Send {
        for value in self.values {
            value.write_to(writer).await?;
        }
        Ok(())
    }

    pub fn get_metadata_value(&self) -> Option<&ScriptDataEcmaArray> {
        if self.values.len() > 1 {
            match &*self.values[1] {
                value if value.as_any().is::<ScriptDataEcmaArray>() => value.as_any().downcast_ref::<ScriptDataEcmaArray>(),
                value if value.as_any().is::<ScriptDataObject>() => {
                    // Assuming ScriptDataObject can be treated as a ScriptDataEcmaArray for your purposes
                    // and you have a way to convert or access it as such.
                    value.as_any().downcast_ref::<ScriptDataEcmaArray>()
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

pub async fn read_script_data_string<R: AsyncRead + Unpin>(reader: &mut R, expect_object_end_marker: bool) -> Result<String> {
    let length = reader.read_u16().await?;
    if length == 0 {
        if expect_object_end_marker && reader.read_u8().await? != 9 {
            return Err(anyhow::anyhow!("ObjectEndMarker not matched."));
        }
        return Ok(String::new());
    }
    let mut buf = BytesMut::with_capacity(length as usize);
    reader.read_exact(&mut buf).await?;
    let string = String::from_utf8(buf.freeze().to_vec())
        .map_err(|e| Err(anyhow::anyhow!("string to utf-8 is error")))?
        .replace("\0", "");
    return Ok(string);
}

pub async fn read_long_string<R: AsyncRead + Unpin>(reader: &mut R) -> Result<String> {
    let length = reader.read_u32().await?;
    if length > u32::MAX {
        return Err(anyhow::anyhow!("LongString larger than {} is not supported.", u32::MAX));
    }

    let mut buf = BytesMut::with_capacity(length as usize);
    reader.read_exact(&mut buf).await?;
    let string = String::from_utf8(buf.freeze().to_vec())
        .map_err(|e| Err(anyhow::anyhow!("string to utf-8 is error")))?
        .replace("\0", "");
    return Ok(string);
}
