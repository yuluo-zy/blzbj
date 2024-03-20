use std::fmt::Display;
use crate::pipline::CommentType;

pub struct ProcessingComment {
    pub comment_type: CommentType,
    pub action_required: bool,
    pub comment: String,
}


impl ProcessingComment {
    pub fn new(comment_type: CommentType, action_required: bool, comment: String) -> Self {
        if comment.is_empty() {
            panic!("comment cannot be empty");
        }
        ProcessingComment {
            comment_type,
            action_required,
            comment,
        }
    }
}

impl Display for ProcessingComment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Rust 中字符串格式化使用 `format!` 宏
        let str = format!(
            "({:?},{:?}): {}",
            self.comment_type,
            if self.action_required { "A" } else { "C" },
            self.comment
        );
        write!(f, "{}", str)
    }
}