mod script_values;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use anyhow::Result;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use crate::amf::script_values::ScriptDataBoolean;

// 定义 ScriptDataType 枚举，匹配 C# 中的 ScriptDataType。
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ScriptDataType {
    Number = 0,
    Boolean = 1,
    String = 2,
    Object = 3,
    MovieClip = 4,
    Null = 5,
    Undefined = 6,
    Reference = 7,
    EcmaArray = 8,
    ObjectEndMarker = 9,
    StrictArray = 10,
    Date = 11,
    LongString = 12,
}

// 定义 ScriptDataValue 枚举，其中包含不同类型的数据。
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "value")]
pub enum ScriptDataValue {
    Number(f64),
    Boolean(ScriptDataBoolean),
    String(String),
    Object(JsonValue),
    Null,
    Undefined,
    Reference(u16),
    EcmaArray(Vec<(String, Box<ScriptDataValue>)>),
    StrictArray(Vec<ScriptDataValue>),
    Date(f64),
    LongString(String),
}

#[async_trait]
pub trait ScriptDataValueTrait {
    fn data_type(&self) -> ScriptDataType;
    async fn write_to<W>(self, writer: &mut W) -> Result<()>
        where
            W: AsyncWrite + Unpin + Send;

    // async fn read_from<R>(reader: &mut R) -> JsonResult<Self>
    //     where
    //         R: AsyncRead + Unpin + Send;

}

// 实现序列化和解序列化行为
impl ScriptDataValueTrait for ScriptDataValue {
    fn data_type(&self) -> ScriptDataType {
        todo!()
    }

    async fn write_to<W>(self, writer: &mut W) -> Result<()>
        where
            W: AsyncWrite + Unpin + Send,
    {
        let serialized_data = serde_json::to_vec(&self)?;
        writer.write_all(&serialized_data).await?;
        Ok(())
    }

    // // 异步读取方法
    // async fn read_from<R>(reader: &mut R) -> Result<Self>
    //     where
    //         R: AsyncRead + Unpin + Send,
    // {
    //     let mut buffer = Vec::new();
    //     reader.read_to_end(&mut buffer).await?;
    //     let value = serde_json::from_slice(&buffer)?;
    //     Ok(value)
    // }
}
