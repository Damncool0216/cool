#![no_std]

pub mod general;
pub mod gnss;
pub mod mqtt;
pub mod client;
pub mod digester;
pub mod urc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
