pub mod client;
pub mod digester;
pub mod general;
pub mod gnss;
pub mod mqtt;
pub mod net;
pub mod urc;

use core::str;
pub(crate) fn parse_num<I: str::FromStr>(data: &str) -> Result<I, &'static str> {
    data.parse::<I>().map_err(|_| "parse of number failed")
}
