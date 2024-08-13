mod live;
mod api;


#[derive(Debug)]
pub struct ApiRequestError {
    pub code: i32,
    pub message: String,
}
