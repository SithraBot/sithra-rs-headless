use ioevent::rpc::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, ProcedureCall)]
pub struct TakeScreenshot {
    pub url: String,
    pub selector: Option<String>,
}

impl ProcedureCallRequest for TakeScreenshot {
    type RESPONSE = TakeScreenshotResponse;
}

#[derive(Serialize, Deserialize, ProcedureCall)]
pub struct TakeScreenshotResponse {
    pub file_path: String,
}
impl ProcedureCallResponse for TakeScreenshotResponse {}
