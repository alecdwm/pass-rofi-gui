use anyhow::Context;
use anyhow::Error;
use miniotp::TOTP;

const OTP_DIGITS: u32 = 6;

pub fn calculate_otp(secret: &str) -> Result<String, Error> {
    TOTP::from_base32(secret)
        .map(|totp| totp.generate_now())
        .map(format_otp)
        .context("Failed to calculate OTP")
}

fn format_otp(otp: u32) -> String {
    format!("{:0width$}", otp, width = OTP_DIGITS as usize)
}

#[cfg(test)]
mod tests {
    use super::format_otp;

    #[test]
    fn test_format_otp_6_digits() {
        let tests = [
            (12345678, "12345678"),
            (123456, "123456"),
            (1234, "001234"),
            (1, "000001"),
            (0, "000000"),
        ];

        for test in &tests {
            assert_eq!(format_otp(test.0), test.1);
        }
    }
}
