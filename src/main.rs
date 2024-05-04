#![no_std]
#![no_main]
#![allow(dead_code)]

use backtrace as _;
use embassy_executor::Spawner;

use hal::prelude::*;
mod apl;
mod fml;
mod pal;

#[main]
async fn main(spawner: Spawner) {
    pal::init(&spawner);
    fml::init(&spawner);
    apl::init(&spawner);
}
