#![no_std]
#![no_main]
#![allow(dead_code)]

use backtrace as _;
use embassy_executor::Spawner;

use hal::prelude::*;
pub mod apl;
pub mod fml;
pub mod pal;

#[main]
async fn main(spawner: Spawner) {
    pal::init(&spawner);
    fml::init(&spawner);
    apl::init(&spawner);
}
