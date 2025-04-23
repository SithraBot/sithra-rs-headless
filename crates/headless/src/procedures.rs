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
    take_screenshot_(&state, request).await
}

pub async fn take_screenshot_(
    state: &State<HeadlessState>,
    request: TakeScreenshot,
) -> Result<TakeScreenshotResponse, CallSubscribeError> {
    let tab = state.browser.new_tab().map_err(into_err)?;
    let TakeScreenshot { url, selector } = request;
    tab.set_user_agent(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36",
        Some("zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7,lt;q=0.6"),
        Some("macOS"),
    ).map_err(into_err)?;
    tab.navigate_to(&url).map_err(into_err)?;
    let selector = if let Some(selector) = selector {
        selector
    } else {
        "body".to_string()
    };
    tab.wait_for_element(&selector).map_err(into_err)?;
    let file_path = format!("./headless/screenshot/screenshot_{}.jpeg", Uuid::new_v4());
    let file_path = std::path::Path::new(&file_path);
    let file_dir = file_path.parent().unwrap();
    fs::create_dir_all(file_dir).await.map_err(into_err)?;
    let element = tab.wait_for_element(&selector).map_err(into_err)?;
    let image = element
        .capture_screenshot(Page::CaptureScreenshotFormatOption::Jpeg)
        .map_err(into_err)?;
    fs::write(file_path, image).await.map_err(into_err)?;
    tab.close(true).map_err(into_err)?;
    let file_path = fs::canonicalize(file_path).await.map_err(into_err)?;
    Ok(TakeScreenshotResponse {
        file_path: file_path.to_string_lossy().into_owned(),
    })
}

fn into_err<E>(e: E) -> CallSubscribeError
where
    E: ToString,
{
    CallSubscribeError::Other(e.to_string())
}
