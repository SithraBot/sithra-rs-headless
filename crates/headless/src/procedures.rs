use headless_chrome::protocol::cdp::Page;
use ioevent::error::CallSubscribeError;
use ioevent::prelude::*;
use ioevent::rpc::*;
use sithra_headless_common::TakeScreenshot;
use sithra_headless_common::TakeScreenshotResponse;
use tokio::fs;
use uuid::Uuid;

use crate::HeadlessState;

#[procedure]
pub async fn take_screenshot(state: State<HeadlessState>, request: TakeScreenshot) -> Result {
    let tab = state.browser.new_tab().map_err(into_err)?;
    let TakeScreenshot { url, selector } = request;
    tab.navigate_to(&url).map_err(into_err)?;
    let file_path = format!("./headless/screenshot_{}.jpeg", Uuid::new_v4());
    let file_path = fs::canonicalize(&file_path)
        .await
        .map_err(into_err)?
        .to_string_lossy()
        .to_string();
    if let Some(selector) = selector {
        let element = tab.wait_for_element(&selector).map_err(into_err)?;
        let image = element
            .capture_screenshot(Page::CaptureScreenshotFormatOption::Jpeg)
            .map_err(into_err)?;
        fs::write(&file_path, image).await.map_err(into_err)?;
        Ok(TakeScreenshotResponse { file_path })
    } else {
        let image = tab
            .capture_screenshot(Page::CaptureScreenshotFormatOption::Jpeg, None, None, false)
            .map_err(into_err)?;
        fs::write(&file_path, image).await.map_err(into_err)?;
        Ok(TakeScreenshotResponse { file_path })
    }
}

fn into_err<E>(e: E) -> CallSubscribeError
where
    E: ToString,
{
    CallSubscribeError::Other(e.to_string())
}
