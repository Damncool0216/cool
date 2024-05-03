#![no_std]
#![no_main]

use embassy_executor::Spawner;
use backtrace as _;

use hal::prelude::*;
mod pal;
mod fml;
mod apl;

#[main]
async fn main(spawner: Spawner) {
    pal::init(&spawner);
    fml::init(&spawner);
    apl::init(&spawner);
}
