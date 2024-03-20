use crate::pipline::processing_comment::ProcessingComment;
use crate::pipline::CommentType;

pub struct HandleDelayedAudioHeaderRule {
    comments: Vec<ProcessingComment>
}

impl Default for HandleDelayedAudioHeaderRule {
    fn default() -> Self {
        let comment1 = ProcessingComment::new(CommentType::Unrepairable, true, "音频数据出现在音频头之前".to_string());

        let comment2 = ProcessingComment::new(CommentType::DecodingHeader, true, "检测到延后收到的音频头".to_string());
        
        HandleDelayedAudioHeaderRule {
            comments: vec![comment1, comment2],
        }
    }
}

impl HandleDelayedAudioHeaderRule {
    pub fn run_per_action() {

    }
}