#[derive(Debug)]
pub enum Notification {
    Success { msg: &'static str },
    Warn { msg: &'static str },
    Error { msg: String },
}
