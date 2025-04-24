mod config;
mod procedures;

use config::Config;
use fantoccini::wd::Capabilities;
use fantoccini::{Client, ClientBuilder};
use parking_lot::Mutex;
use procedures::*;
use serde_json::json;
use sithra_headless_common::TakeScreenshot;
use triomphe::Arc;

use std::fs;
use std::path::Path;

use ioevent::{prelude::*, rpc::*};
use sithra_common::event::MessageEventFlattened as Message;
use sithra_common::prelude::*;

const SUBSCRIBER: &[Subscriber<HeadlessState>] = &[
    create_subscriber!(take_screenshot),
    create_subscriber!(take_screenshot_subscriber),
];

#[derive(Clone)]
struct HeadlessState {
    browser: Arc<Mutex<Client>>,
    self_id: u64,
    pcw: DefaultProcedureWright,
}

impl SithraState for HeadlessState {
    fn self_id(&self) -> u64 {
        self.self_id
    }
    async fn create(self_id: u64) -> Self {
        if !Path::new("./headless").exists() {
            fs::create_dir_all("./headless").unwrap();
        }
        let path = fs::canonicalize("./headless").unwrap();
        let config = Config::init(path).await;
        let mut browser = ClientBuilder::native();
        let mut capabilities = Capabilities::new();
        let firefox_options = json! ({
            "args": ["-headless"],
        });
        let chrome_options = json! ({
            "args": ["--headless"],
        });
        capabilities.insert("moz:firefoxOptions".to_string(), firefox_options);
        capabilities.insert("goog:chromeOptions".to_string(), chrome_options);
        browser.capabilities(capabilities);
        Self {
            browser: Arc::new(Mutex::new(
                browser.connect(&config.webdriver_url).await.unwrap(),
            )),
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
