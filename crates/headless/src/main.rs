mod procedures;

use headless_chrome::browser::context::Context;
use procedures::*;
use sithra_headless_common::TakeScreenshot;

use std::fs;

use headless_chrome::{Browser, LaunchOptions};
use ioevent::{prelude::*, rpc::*};
use sithra_common::event::MessageEventFlattened as Message;
use sithra_common::prelude::*;

const SUBSCRIBER: &[Subscriber<HeadlessState>] = &[
    create_subscriber!(take_screenshot),
    create_subscriber!(take_screenshot_subscriber),
];

#[derive(Clone)]
struct HeadlessState {
    browser: Browser,
    self_id: u64,
    pcw: DefaultProcedureWright,
}

impl SithraState for HeadlessState {
    fn self_id(&self) -> u64 {
        self.self_id
    }
    fn create(self_id: u64) -> Self {
        let path = fs::canonicalize("./headless").unwrap();
        Self {
            browser: Browser::new(
                LaunchOptions::default_builder()
                    .enable_logging(false)
                    .window_size(None)
                    .sandbox(false)
                    .user_data_dir(Some(path))
                    .idle_browser_timeout(std::time::Duration::from_secs(60 * 30))
                    .build()
                    .unwrap(),
            )
            .unwrap(),
            self_id,
            pcw: DefaultProcedureWright::default(),
        }
    }
}

impl ProcedureCallWright for HeadlessState {
    fn next_echo(&self) -> impl Future<Output = u64> + Send + Sync {
        self.pcw.next_echo()
    }
}

#[subscriber]
async fn take_screenshot_subscriber(state: State<HeadlessState>, msg: Message) -> Result {
    if !msg.starts_with("take ") {
        return Ok(());
    }
    if msg.len() != 1 {
        return Ok(());
    }
    let url = if let Some(MessageNode::Text(url)) = msg.first() {
        url.trim_start_matches("take ")
    } else {
        return Ok(());
    };
    let requset = TakeScreenshot {
        url: url.to_string(),
        selector: Some("html".to_string()),
    };
    let img = take_screenshot_(&state, requset).await?;
    let file_path = img.file_path;
    let file_url = format!("file://{}", file_path);
    let reply_msg = vec![MessageNode::Image(file_url)];
    msg.reply(&state, reply_msg).await?;
    Ok(())
}

#[sithra_common::main(subscribers = SUBSCRIBER, state = HeadlessState)]
async fn main(_ew: &EffectWright) {
    fs::create_dir_all("./headless").unwrap();
}
