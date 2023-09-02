#[derive(Clone, Debug)]
pub struct UserMessage {
    pub user: String,
    pub message: String,
}

impl UserMessage {
    pub fn new(user: String, message: String) -> Self {
        Self {
            user,
            message,
        }
    }
}