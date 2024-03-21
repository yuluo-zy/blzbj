mod tag;
mod h264_nalu;
mod reader;
mod writer;
mod group_rule;
mod group_reader;
mod pipline;
mod amf;
mod error;
mod borrow_bag;
mod group;
mod parser;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
