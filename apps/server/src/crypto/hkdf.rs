use hkdf::Hkdf;
use sha2::Sha256;

pub fn derive_key(
    ikm: &[u8],
    salt: &[u8],
    info: &[u8],
    out: &mut [u8],
) -> Result<(), hkdf::InvalidLength> {
    let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    hk.expand(info, out)?;
    Ok(())
}
