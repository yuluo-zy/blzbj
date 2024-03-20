mod script_values;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use num_enum::TryFromPrimitive;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use crate::amf::script_values::{ScriptDataBoolean, ScriptDataDate, ScriptDataEcmaArray, ScriptDataLongString, ScriptDataNull, ScriptDataNumber, ScriptDataObject, ScriptDataReference, ScriptDataStrictArray, ScriptDataString, ScriptDataUndefined};

// 定义 ScriptDataType 枚举，匹配 C# 中的 ScriptDataType。
#[derive(Serialize, Deserialize, Debug, PartialEq, TryFromPrimitive)]
#[serde(rename_all = "camelCase")]
#[repr(u8)]
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
    Number(ScriptDataNumber),
    Boolean(ScriptDataBoolean),
    String(ScriptDataString),
    Object(ScriptDataObject),
    Null(ScriptDataNull),
    Undefined(ScriptDataUndefined),
    Reference(ScriptDataReference),
    EcmaArray(ScriptDataEcmaArray),
    StrictArray(ScriptDataStrictArray),
    Date(ScriptDataDate),
    LongString(ScriptDataLongString),
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
