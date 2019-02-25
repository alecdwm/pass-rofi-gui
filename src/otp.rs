use boringauth::oath::TOTPBuilder;
use failure::format_err;
use failure::Error;

pub fn calculate_otp(secret: &str) -> Result<String, Error> {
    Ok(TOTPBuilder::new()
        .base32_key(&secret.to_owned())
        .finalize()
        .map_err(|err| format_err!("BoringAuth error: {:?}", err))?
        .generate())
}
