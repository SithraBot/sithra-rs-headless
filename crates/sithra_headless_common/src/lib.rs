use ioevent::rpc::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, ProcedureCall)]
pub struct TakeScreenshot {
    pub preprocess_script: Option<String>,
    pub url: String,
    pub selector: Option<String>,
}

impl ProcedureCallRequest for TakeScreenshot {
    type RESPONSE = TakeScreenshotResponse;
}

#[derive(Serialize, Deserialize, ProcedureCall)]
pub enum TakeScreenshotResponse {
    Success(String),
    Err(ErrKind),
}
impl ProcedureCallResponse for TakeScreenshotResponse {}

#[derive(Serialize, Deserialize)]
pub enum ErrKind {
    Timeout,
    Other(String),
}
