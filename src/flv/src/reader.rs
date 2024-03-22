// use std::io;
// use std::io::Cursor;
// use anyhow::{Error, Result};
// use bytes::{Buf, BufMut, BytesMut};
// use nom::{IResult, Needed};
// use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
// use crate::error::TagReaderError;
// use crate::error::TagReaderError::ParseFileHeaderError;
// use crate::parser::{complete_tag, header};
//
// use crate::tag::{Header, Tag, TagType};
//
// pub struct FlvTagPipeReader<R: AsyncRead + Unpin> {
//     file_header: bool,
//     buffer: BytesMut,
//     stream: BufReader<R>,
//     index: usize
// }
//
// impl<R: AsyncRead + Unpin> FlvTagPipeReader<R>  {
//     pub fn new(r: R) -> Result<Self> {
//         Ok(FlvTagPipeReader {
//             file_header: false,
//             buffer: BytesMut::with_capacity(4 * 1024),
//             stream: BufReader::new(r),
//             index: 0
//         })
//     }
//
//     pub fn read_next_tag(&mut self) -> Result<Tag, TagReaderError> {
//             // 试图解析文件头
//         // let mut buf = Cursor::new(&self.buffer[..]);
//             let mut index = 0;
//             if self.file_header {
//                 if self.buffer.remaining() < 9 {
//                     return Err(TagReaderError::Incomplete);
//                 }
//
//                 match header(&self.buffer[..9]) {
//                     Ok(_) => {
//                         self.file_header = true;
//                         index = 9;
//                     }
//                     Err(e) => {
//                        return Err(TagReaderError::ParseFileHeaderError(e.to_string()))
//                     }
//                 }
//             }
//
//
//         if self.buffer.remaining() < index + 4 { return Err(TagReaderError::Incomplete); }
//
//         return match complete_tag(&self.buffer[index + 4..]) {
//             Ok((remaining, parsed_data)) => {
//                 self.index = index + (self.buffer.remaining() -remaining.len());
//                 return Ok( parsed_data)
//             }
//             Err(nom::Err::Incomplete(_)) => {
//                 Err(TagReaderError::Incomplete)
//             }
//             Err(e) => {
//                 Err(TagReaderError::ParseTagError(e.to_string()))
//             }
//         }
//     }
//
//     pub async fn read_tag(&mut self) -> Result<Option<Tag>> {
//         loop {
//              let tag = self.read_next_tag();
//             if let Ok(tag_data) = tag{
//                 self.buffer.advance(self.index);
//                 return Ok(Some(tag_data));
//             }
//
//         }
//     }
//
//
// }
//
//
//
// // #[cfg(test)]
// // mod tests {
// //     use std::sync::Arc;
// //     use super::*;
// //     use tokio::fs::File;
// //
// //     fn parse_data(buffer: &mut BytesMut) -> io::Result<Option<Tag>> {
// //         // 解析逻辑...
// //         // 返回处理的字节数和可能的解析数据
// //         Ok(None)// 示例，实际应根据解析逻辑返回
// //     }
// //
// //     // 异步读取和解析数据的循环
// //     #[tokio::test]
// //     async fn read_and_parse() -> io::Result<()> {
// //         let mut file = File::open("../assets/test.flv").await?;
// //         let mut buffer = BytesMut::with_capacity(4096);
// //
// //         let mut message: usize = 0;
// //         let mut bytes_transferred: usize = 0;
// //         let mut buf = BytesMut::with_capacity(1024);
// //         loop {
// //             let ciphertext_len = file.read_buf(&mut buf).await?;
// //             if ciphertext_len == 0 {
// //                 break;
// //             } else if buf.len() == 1024 {
// //                 message += 1;
// //                 buf.clear();
// //             }
// //         }
// //
// //         Ok(())
// //     }
// //
// //     #[tokio::test]
// //     async fn test_read_next_tag() -> Result<()> {
// //         // 创建模拟的 AsyncRead，返回预定义的 FLV 数据
// //         let mut file = File::open("../assets/test.flv").await?;
// //         let mut reader = FlvTagPipeReader::new()?;
// //         let mut tags = Vec::new();
// //         let mut buffer = BytesMut::with_capacity(4096);
// //         loop {
// //
// //             match  reader.read_next_tag( &buffer[..]) {
// //                 Ok((advance_by, parsed_data)) => {
// //                     // 现在可以安全地可变借用 buffer 了，因为 buffer_slice 已经被 drop
// //                     let advance_by = buffer.len() - advance_by.len();
// //                     tags.push(parsed_data);
// //                     // 现在可以安全地可变借用 buffer 了
// //                     buffer.advance(advance_by);
// //                 }
// //                 Err(TagReaderError::Incomplete) => {}
// //                 Err(e) => {
// //                     // 传播其他错误
// //                     return Err(e.into());
// //                 }
// //             }
// //             let num = { file.read_buf(&mut buffer).await? };
// //             if num == 0 { break;  }
// //
// //
// //
// //
// //             // 如果我们已经处理了 buffer 中的所有数据，那么可以清空 buffer，以准备下一次读取
// //             if buffer.is_empty() {
// //                 buffer.clear();
// //             }
// //         }
// //
// //
// //         // 测试断言：确保解析出正确数量的标签
// //         // 这里应该根据你的 FLV 文件内容来调整
// //         // assert!(!tag.is_some(), "No tags were parsed from the file.");
// //
// //         Ok(())
// //     }
// // }
