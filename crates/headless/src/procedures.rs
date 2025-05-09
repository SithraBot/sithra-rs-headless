use std::path::Path;
use std::time::Duration;

use ioevent::error::CallSubscribeError;
use ioevent::prelude::*;
use ioevent::rpc::*;
use sithra_headless_common::ErrKind;
use sithra_headless_common::TakeScreenshot;
use sithra_headless_common::TakeScreenshotResponse;
use tokio::fs;
use tokio::time::Instant;
use tokio::time::timeout;
use uuid::Uuid;

use crate::HeadlessState;

#[procedure]
pub async fn take_screenshot(state: State<HeadlessState>, request: TakeScreenshot) -> Result {
    take_screenshot_(&state, request).await
}

macro_rules! return_err {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => return Ok(TakeScreenshotResponse::Err(ErrKind::Other(e.to_string()))),
        }
    };
}

pub async fn take_screenshot_(
    state: &State<HeadlessState>,
    request: TakeScreenshot,
) -> Result<TakeScreenshotResponse, CallSubscribeError> {
    let TakeScreenshot {
        url,
        selector,
        preprocess_script,
    } = request;
    let browser = state.browser.lock();
    return_err!(browser.set_window_rect(0, 0, 640, 480).await);
    return_err!(return_err!(
        timeout(Duration::from_secs(30), browser.goto(&url)).await
    ));
    let current = return_err!(browser.current_url().await);
    return_err!(browser.wait().for_url(current).await);
    let start = Instant::now();
    loop {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let ready_state = return_err!(browser.execute("return document.readyState", vec![]).await);
        let ready_state = ready_state.as_str();
        if ready_state == Some("complete") {
            break;
        }
        if start.elapsed() > Duration::from_secs(45) {
            break;
        }
    }
    let selector = if let Some(selector) = selector {
        selector
    } else {
        "body".to_string()
    };
    if let Some(preprocess_script) = preprocess_script {
        return_err!(browser.execute(&preprocess_script, vec![]).await);
    }
    let element = return_err!(
        browser
            .wait()
            .for_element(fantoccini::Locator::Css(&selector))
            .await
    );
    let w = return_err!(
        browser
            .execute("return document.documentElement.scrollWidth", vec![])
            .await
    );
    let h = return_err!(
        browser
            .execute("return document.documentElement.scrollHeight", vec![])
            .await
    );
    let w = w.as_f64().unwrap();
    let h = h.as_f64().unwrap();
    return_err!(browser.set_window_size(w as u32, h as u32).await);
    // let (_, _, w, h) = return_err!(element.rectangle().await);
    // return_err!(browser.set_window_size(w as u32, h as u32).await);
    let screenshot = return_err!(element.screenshot().await);
    let path = Path::new("./headless/screenshots");
    return_err!(fs::create_dir_all(path).await);
    let file_name = format!("{}.png", Uuid::new_v4());
    let file_path = path.join(file_name);
    return_err!(fs::write(&file_path, screenshot).await);
    let file_path = return_err!(fs::canonicalize(file_path).await);
    Ok(TakeScreenshotResponse::Success(
        file_path.to_string_lossy().to_string(),
    ))
}
