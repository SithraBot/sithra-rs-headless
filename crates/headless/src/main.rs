mod procedures;

use procedures::*;

use std::fs;

use headless_chrome::{Browser, LaunchOptions};
use ioevent::{prelude::*, rpc::*};
use sithra_common::prelude::*;

const SUBSCRIBER: &[Subscriber<HeadlessState>] = &[create_subscriber!(take_screenshot)];

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
        Self {
            browser: Browser::new(
                LaunchOptions::default_builder()
                    .sandbox(false)
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

#[sithra_common::main(subscribers = SUBSCRIBER, state = HeadlessState)]
async fn main(_ew: &EffectWright) {
    fs::create_dir_all("headless").unwrap();
}
