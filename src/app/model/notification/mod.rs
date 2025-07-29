use std::borrow::Cow;

#[derive(Debug)]
pub enum Notification {
    Success { msg: Cow<'static, str> },
    Warn { msg: Cow<'static, str> },
    Error { msg: Cow<'static, str> },
}
