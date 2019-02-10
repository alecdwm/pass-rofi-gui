use boringauth::oath::{ErrorCode, TOTPBuilder};

pub fn calculate_otp(secret: &str) -> Result<String, ErrorCode> {
    Ok(TOTPBuilder::new()
        .base32_key(&secret.to_owned())
        .finalize()?
        .generate())
}
