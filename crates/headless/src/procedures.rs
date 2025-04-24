use std::path::Path;

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
    let TakeScreenshot { url, selector } = request;
    let browser = state.browser.lock();
    browser.goto(&url).await.map_err(into_err)?;
    let current_url = browser.current_url().await.map_err(into_err)?;
    if url != current_url.to_string() {
        return Err(into_err(format!(
            "url not match: {} != {}",
            url, current_url
        )));
    }
    let selector = if let Some(selector) = selector {
        selector
    } else {
        "body".to_string()
    };
    let element = browser
        .find(fantoccini::Locator::Css(&selector))
        .await
        .map_err(into_err)?;
    let (_, _, w, h) = element.rectangle().await.map_err(into_err)?;
    browser
        .set_window_size(w as u32, h as u32)
        .await
        .map_err(into_err)?;
    let screenshot = element.screenshot().await.map_err(into_err)?;
    let path = Path::new("./headless/screenshots");
    fs::create_dir_all(path).await.map_err(into_err)?;
    let file_name = format!("{}.png", Uuid::new_v4());
    let file_path = path.join(file_name);
    fs::write(&file_path, screenshot).await.map_err(into_err)?;
    let file_path = fs::canonicalize(file_path).await.map_err(into_err)?;
    Ok(TakeScreenshotResponse {
        file_path: file_path.to_string_lossy().to_string(),
    })
}

fn into_err<E>(e: E) -> CallSubscribeError
where
    E: ToString,
{
    CallSubscribeError::Other(e.to_string())
}
