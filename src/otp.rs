use anyhow::anyhow;
use anyhow::Error;
use libotp::totp;

const OTP_DIGITS: u32 = 6;
const OTP_TIME_STEP: u64 = 30;
const OTP_TIME_START: u64 = 0;

pub fn calculate_otp(secret: &str) -> Result<String, Error> {
    totp(secret, OTP_DIGITS, OTP_TIME_STEP, OTP_TIME_START)
        .map(|otp| format_otp(otp))
        .ok_or_else(|| anyhow!("Failed to calculate OTP"))
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
