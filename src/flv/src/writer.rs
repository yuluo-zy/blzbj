use tokio::io::{AsyncWrite, AsyncWriteExt};
use anyhow::Result;
use bytes::BytesMut;

pub struct FlvWriterMuxer<W: AsyncWrite + AsyncWriteExt + Unpin> {
    pub writer: W,
}

impl<W> FlvWriterMuxer<W> where W: AsyncWrite + AsyncWriteExt + Unpin {
    pub fn new(writer: W) -> Self {
        Self {
            writer
        }
    }

    pub async fn write_u24(&mut self, value: u32) -> Result<()> {
        let buf = &value.to_be_bytes()[1..];
        self.writer.write_all(buf).await?;
        Ok(())
    }
    pub async fn write_flv_header(&mut self, tag_type: u8,
                                  data_size: u32,
                                  timestamp: u32, ) -> Result<()> {
        //tag type
        self.writer.write_u8(tag_type).await?;
        //data size
        self.write_u24(data_size).await?;
        //timestamp
        self.write_u24(timestamp & 0xffffff).await?;
        //timestamp extended.
        let timestamp_ext = (timestamp >> 24 & 0xff) as u8;
        self.writer.write_u8(timestamp_ext).await?;
        //stream id
        self.write_u24(0).await?;

        Ok(())
    }

    pub async fn write_flv_tag_body(&mut self, body: BytesMut) -> Result<()> {
        self.writer.write(&body[..]).await?;
        Ok(())
    }

    pub async fn write_previous_tag_size(&mut self, size: u32) -> Result<()> {
        self.writer.write_u32(size).await?;
        Ok(())
    }
}