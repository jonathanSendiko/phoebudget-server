use serde::Serialize;

use crate::i18n;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: Option<String>) -> Self {
        Self {
            success: true,
            data,
            message: message.map(|msg| i18n::localize_message(&msg)),
        }
    }
}
