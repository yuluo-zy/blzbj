use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait TagGroupReader {
    // 异步读取一个标签组，并可能产生一个管道行动。
    // async fn read_group_async(&mut self) -> Result<Option<PipelineAction>>;
}