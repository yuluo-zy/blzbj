// use async_recursion::async_recursion;
// use indexmap::IndexMap;
// use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader};
// use crate::error::Amf0ReadError;
// use anyhow::Result;
// use bytes::BytesMut;
//
// pub const NUMBER: u8 = 0x00;
// pub const BOOLEAN: u8 = 0x01;
// pub const STRING: u8 = 0x02;
// pub const OBJECT: u8 = 0x03;
// pub const NULL: u8 = 0x05;
// pub const ECMA_ARRAY: u8 = 0x08;
// pub const OBJECT_END: u8 = 0x09;
// pub const LONG_STRING: u8 = 0x0c;
//
// #[derive(PartialEq, Clone, Debug)]
// pub enum Amf0ValueType {
//     Number(f64),
//     Boolean(bool),
//     UTF8String(String),
//     Object(IndexMap<String, Amf0ValueType>),
//     Null,
//     EcmaArray(IndexMap<String, Amf0ValueType>),
//     LongUTF8String(String),
//     END,
// }
//
//
// pub struct Amf0Reader<R: AsyncRead + AsyncReadExt + Unpin + Send> {
//     reader: BufReader<R>,
// }
//
// async fn read_u24<R>(reader: &mut R) -> Result<u32, Amf0ReadError>
//     where
//         R: AsyncReadExt + Unpin,
// {
//     let mut bytes = [0u8; 3];
//     reader.read_exact(&mut bytes).await?;
//
//     let num = u32::from(bytes[0]) << 16 | u32::from(bytes[1]) << 8 | u32::from(bytes[2]);
//
//     Ok(num)
// }
//
// impl<R: AsyncRead + AsyncReadExt + Unpin + Send> Amf0Reader<R> {
//     pub fn new(reader: R) -> Self {
//         Self { reader: BufReader::new(reader) }
//     }
//     pub async fn read_all(&mut self) -> Result<Vec<Amf0ValueType>, Amf0ReadError> {
//         let mut results = vec![];
//
//         loop {
//             let result = self.read_any().await?;
//             match result {
//                 Amf0ValueType::END => {
//                     break;
//                 }
//                 _ => {
//                     results.push(result);
//                 }
//             }
//         }
//         Ok(results)
//     }
//     #[async_recursion]
//     pub async fn read_any(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
//         if self.reader.buffer().is_empty() {
//             return Ok(Amf0ValueType::END);
//         }
//         let markers = self.reader.read_u8().await?;
//
//         if markers == OBJECT_END {
//             return Ok(Amf0ValueType::END);
//         }
//
//         match markers {
//             NUMBER => self.read_number().await,
//             BOOLEAN => self.read_bool().await,
//             STRING => self.read_string().await,
//             OBJECT => self.read_object().await,
//             NULL => self.read_null(),
//             ECMA_ARRAY => self.read_ecma_array().await,
//             LONG_STRING => self.read_long_string().await,
//             _ => Err(Amf0ReadError::UnknownMarker(markers)),
//         }
//     }
//
//     pub async fn read_number(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
//         let number = self.reader.read_f64().await?;
//         let value = Amf0ValueType::Number(number);
//         Ok(value)
//     }
//
//     pub async fn read_bool(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
//         let value = self.reader.read_u8().await?;
//
//         match value {
//             1 => Ok(Amf0ValueType::Boolean(true)),
//             _ => Ok(Amf0ValueType::Boolean(false)),
//         }
//     }
//
//     pub async fn read_raw_string(&mut self) -> Result<String, Amf0ReadError> {
//         let l = self.reader.read_u16().await?;
//         let mut bytes: Vec<u8> = vec![0u8; l as usize];
//         self.reader.read_exact(&mut bytes).await?;
//         let val = String::from_utf8(bytes.to_vec())?;
//
//         Ok(val)
//     }
//
//     pub async fn read_string(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
//         let raw_string = self.read_raw_string().await?;
//         Ok(Amf0ValueType::UTF8String(raw_string))
//     }
//
//     pub fn read_null(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
//         Ok(Amf0ValueType::Null)
//     }
//
//     pub async fn is_read_object_eof(&mut self) -> Result<bool, Amf0ReadError> {
//         let buf = self.reader.fill_buf().await?;
//         if buf.len() > 3 {
//             let result = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
//             if result == OBJECT_END as u32 {
//                 read_u24(&mut self.reader).await?;
//                 return Ok(true);
//             }
//         }
//         Ok(false)
//     }
//
//     pub async fn read_with_type(&mut self, specified_marker: u8) -> Result<Amf0ValueType, Amf0ReadError> {
//         let buf = self.reader.fill_buf().await?;
//         if buf.len() > 1 {
//             if buf[0] != specified_marker {
//                 return Err(Amf0ReadError::WrongType);
//             }
//         }
//         self.read_any().await
//     }
//     pub async fn read_object(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
//         let mut properties = IndexMap::new();
//
//         loop {
//             let is_eof = self.is_read_object_eof().await?;
//
//             if is_eof {
//                 break;
//             }
//
//             let key = self.read_raw_string().await?;
//             let val = self.read_any().await?;
//
//             properties.insert(key, val);
//         }
//
//         Ok(Amf0ValueType::Object(properties))
//     }
//
//     pub async fn read_ecma_array(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
//         let len = self.reader.read_u32().await?;
//
//         let mut properties = IndexMap::new();
//
//         //here we do not use length to traverse the map, because in some
//         //other media server, the length is 0 which is not correct.
//         while !self.is_read_object_eof().await? {
//             let key = self.read_raw_string().await?;
//             let val = self.read_any().await?;
//             properties.insert(key, val);
//         }
//
//         if len != properties.len() as u32 {
//             log::warn!("the ecma array length is not correct!");
//         }
//
//         Ok(Amf0ValueType::Object(properties))
//     }
//
//     pub async fn read_long_string(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
//         let l = self.reader.read_u32().await?;
//         let mut bytes: Vec<u8> = Vec::with_capacity(l as usize);
//         self.reader.read_exact(&mut bytes).await?;
//         let val = String::from_utf8(bytes)?;
//         Ok(Amf0ValueType::LongUTF8String(val))
//     }
// }
//
//
// #[cfg(test)]
// mod tests {
//     use std::io::Cursor;
//     use super::*;
//
//     use indexmap::IndexMap;
//
//     #[tokio::test]
//     async fn test_amf_reader() {
//         let data: [u8; 177] = [
//             2, 0, 7, 99, 111, 110, 110, 101, 99, 116, 0, 63, 240, 0, 0, 0, 0, 0, 0, //body
//             3, 0, 3, 97, 112, 112, 2, 0, 6, 104, 97, 114, 108, 97, 110, 0, 4, 116, 121, 112, 101,
//             2, 0, 10, 110, 111, 110, 112, 114, 105, 118, 97, 116, 101, 0, 8, 102, 108, 97, 115,
//             104, 86, 101, 114, 2, 0, 31, 70, 77, 76, 69, 47, 51, 46, 48, 32, 40, 99, 111, 109, 112,
//             97, 116, 105, 98, 108, 101, 59, 32, 70, 77, 83, 99, 47, 49, 46, 48, 41, 0, 6, 115, 119,
//             102, 85, 114, 108, 2, 0, 28, 114, 116, 109, 112, 58, 47, 47, 108, 111, 99, 97, 108,
//             104, 111, 115, 116, 58, 49, 57, 51, 53, 47, 104, 97, 114, 108, 97, 110, 0, 5, 116, 99,
//             85, 114, 108, 2, 0, 28, 114, 116, 109, 112, 58, 47, 47, 108, 111, 99, 97, 108, 104,
//             111, 115, 116, 58, 49, 57, 51, 53, 47, 104, 97, 114, 108, 97, 110, 0, 0, 9,
//         ];
//
//
//         let mut amf_reader = Amf0Reader::new(Cursor::new(data));
//
//         let command_name = amf_reader.read_with_type(STRING).await.unwrap();
//         assert_eq!(
//             command_name,
//             Amf0ValueType::UTF8String(String::from("connect"))
//         );
//
//         let transaction_id = amf_reader.read_with_type(NUMBER).await.unwrap();
//         assert_eq!(transaction_id, Amf0ValueType::Number(1.0));
//
//         let command_obj_raw = amf_reader.read_with_type(OBJECT).await.unwrap();
//         let mut properties = IndexMap::new();
//         properties.insert(
//             String::from("app"),
//             Amf0ValueType::UTF8String(String::from("harlan")),
//         );
//         properties.insert(
//             String::from("type"),
//             Amf0ValueType::UTF8String(String::from("nonprivate")),
//         );
//         properties.insert(
//             String::from("flashVer"),
//             Amf0ValueType::UTF8String(String::from("FMLE/3.0 (compatible; FMSc/1.0)")),
//         );
//         properties.insert(
//             String::from("swfUrl"),
//             Amf0ValueType::UTF8String(String::from("rtmp://localhost:1935/harlan")),
//         );
//         properties.insert(
//             String::from("tcUrl"),
//             Amf0ValueType::UTF8String(String::from("rtmp://localhost:1935/harlan")),
//         );
//         assert_eq!(command_obj_raw, Amf0ValueType::Object(properties));
//
//         let _ = amf_reader.read_all().await;
//
//         print!("test")
//     }
//
//     #[tokio::test]
//     async fn test_player_connect_reader() {
//         // chunk header
//         // 0000   03 00 00 00 00 00 aa 14 00 00 00 00
//         //amf0 data
//         //                                            02 00 07 63  ...............c
//         // 0010   6f 6e 6e 65 63 74 00 3f f0 00 00 00 00 00 00 03  onnect.?........
//         // 0020   00 03 61 70 70 02 00 04 6c 69 76 65 00 05 74 63  ..app...live..tc
//         // 0030   55 72 6c 02 00 1a 72 74 6d 70 3a 2f 2f 6c 6f 63  Url...rtmp://loc
//         // 0040   61 6c 68 6f 73 74 3a 31 39 33 35 2f 6c 69 76 65  alhost:1935/live
//         // 0050   00 04 66 70 61 64 01 00 00 0c 63 61 70 61 62 69  ..fpad....capabi
//         // 0060   6c 69 74 69 65 73 00 40 2e 00 00 00 00 00 00 00  lities.@........
//         // 0070   0b 61 75 64 69 6f 43 6f 64 65 63 73 00 40 a8 ee  .audioCodecs.@..
//         // 0080   00 00 00 00 00 00 0b 76 69 64 65 6f 43 6f 64 65  .......videoCode 118 105
//         // 0090   63 73 00 40 6f 80 00 00 00 00 00 00 0d 76 69 64  cs.@o........vid
//         // 00a0   65 6f 46 75 6e 63 74 69 6f 6e 00 3f f0 00 00 00  eoFunction.?....
//         // 0b00   00 00 00 00 00 09                                ......
//
//         let data: [u8; 170] = [
//             0x02, 0x00, 0x07, 0x63, 0x6f, 0x6e, 0x6e, 0x65, 0x63, 0x74, 0x00, 0x3f, 0xf0, 0x00,
//             0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x03, 0x61, 0x70, 0x70, 0x02, 0x00, 0x04,
//             0x6c, 0x69, 0x76, 0x65, 0x00, 0x05, 0x74, 0x63, 0x55, 0x72, 0x6c, 0x02, 0x00, 0x1a,
//             0x72, 0x74, 0x6d, 0x70, 0x3a, 0x2f, 0x2f, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f,
//             0x73, 0x74, 0x3a, 0x31, 0x39, 0x33, 0x35, 0x2f, 0x6c, 0x69, 0x76, 0x65, 0x00, 0x04,
//             0x66, 0x70, 0x61, 0x64, 0x01, 0x00, 0x00, 0x0c, 0x63, 0x61, 0x70, 0x61, 0x62, 0x69,
//             0x6c, 0x69, 0x74, 0x69, 0x65, 0x73, 0x00, 0x40, 0x2e, 0x00, 0x00, 0x00, 0x00, 0x00,
//             0x00, 0x00, 0x0b, 0x61, 0x75, 0x64, 0x69, 0x6f, 0x43, 0x6f, 0x64, 0x65, 0x63, 0x73,
//             0x00, 0x40, 0xa8, 0xee, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0b, 0x76, 0x69, 0x64,
//             0x65, 0x6f, 0x43, 0x6f, 0x64, 0x65, 0x63, 0x73, 0x00, 0x40, 0x6f, 0x80, 0x00, 0x00,
//             0x00, 0x00, 0x00, 0x00, 0x0d, 0x76, 0x69, 0x64, 0x65, 0x6f, 0x46, 0x75, 0x6e, 0x63,
//             0x74, 0x69, 0x6f, 0x6e, 0x00, 0x3f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//             0x00, 0x09,
//         ];
//
//         //76 69 64 65 6f 43 6f 64 65 63 73
//         // 118 105 100 101  111 67 111 100 101 99 115
//         let mut amf_reader = Amf0Reader::new(Cursor::new(data));
//
//         let command_name = amf_reader.read_with_type(STRING).await.unwrap();
//
//         assert_eq!(
//             command_name,
//             Amf0ValueType::UTF8String(String::from("connect"))
//         );
//
//         let transaction_id = amf_reader.read_with_type(NUMBER).await.unwrap();
//         assert_eq!(transaction_id, Amf0ValueType::Number(1.0));
//
//         let command_obj_raw = amf_reader.read_with_type(OBJECT).await;
//
//         if let Err(err) = &command_obj_raw {
//             println!("adfa{err}");
//         }
//
//         let mut properties = IndexMap::new();
//
//         properties.insert(String::from("audioCodecs"), Amf0ValueType::Number(3191.0));
//         properties.insert(String::from("videoCodecs"), Amf0ValueType::Number(252.0));
//         properties.insert(String::from("videoFunction"), Amf0ValueType::Number(1.0));
//         properties.insert(
//             String::from("tcUrl"),
//             Amf0ValueType::UTF8String(String::from("rtmp://localhost:1935/live")),
//         );
//
//         properties.insert(
//             String::from("app"),
//             Amf0ValueType::UTF8String(String::from("live")),
//         );
//
//         properties.insert(String::from("fpad"), Amf0ValueType::Boolean(false));
//         properties.insert(String::from("capabilities"), Amf0ValueType::Number(15.0));
//
//         // let result = amf_writer.write_any(&Amf0ValueType::Object(properties));
//
//         // print::printu8(amf_writer.get_current_bytes());
//
//         assert_eq!(command_obj_raw.unwrap(), Amf0ValueType::Object(properties));
//     }
// }