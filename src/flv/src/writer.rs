// use anyhow::Result;
// use tokio::io::AsyncWrite;
// use crate::tag::Tag;
//
// #[async_trait::async_trait]
// pub trait FlvTagWriter: Send + Sync {
//     // 获取文件大小
//     fn file_size(&self) -> u64;
//
//     // 获取状态
//     // fn state(&self) -> Option<Arc<Mutex<dyn Any + Send>>>;
//
//     // 创建新文件
//     async fn create_new_file(&mut self) -> Result<()>;
//
//     // 关闭当前文件
//     fn close_current_file(&mut self) -> bool;
//
//     // 写入一个标签
//     async fn write_tag(&mut self, tag: Tag) -> Result<()>;
//
//     // 重写元数据
//     // async fn overwrite_metadata(&mut self, metadata: ScriptTagBody) -> io::Result<()>;
//
//     // 写入附带的文本日志
//     async fn write_accompanying_text_log(&mut self, last_tag_duration: f64, message: String) -> Result<()>;
// }
//
// // FlvWriterTargetProvider trait 定义了输出流的创建逻辑。
// #[async_trait::async_trait]
// pub trait FlvWriterTargetProvider {
//     type Writer: AsyncWrite + Send + Unpin + 'static;
//
//     // 异步创建用于写入 FLV 文件数据的输出流。
//     async fn create_output_stream(&self) -> Result<(Self::Writer, Option<Box<dyn Send>>)>;
//
//     // 异步创建用于写入附加文本日志的输出流。
//     async fn create_accompanying_text_log_stream(&self) -> Result<Self::Writer>;
// }
