pub mod cli;
pub mod otp;
pub mod pass;
pub mod pinentry;
pub mod rofi;
pub mod xorg;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
