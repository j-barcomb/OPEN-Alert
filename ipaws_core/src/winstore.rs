//! Windows Certificate Store lookup.
//!
//! On Windows, retrieves a certificate by SHA-1 thumbprint from
//! `CurrentUser\My`, then re-exports it as PKCS#12 so `reqwest`'s
//! `Identity::from_pkcs12_der` can consume it. The exported PKCS#12 is
//! protected by an ephemeral password held in memory only.
//!
//! On non-Windows platforms this module exposes a stub that returns an
//! error so callers don't need `cfg` gates everywhere.

#[derive(Debug)]
pub struct ExportedPkcs12 {
    pub bytes:    Vec<u8>,
    pub password: String,
}

#[cfg(windows)]
pub fn export_pkcs12_by_thumbprint(thumbprint: &str) -> Result<ExportedPkcs12, String> {
    use schannel::cert_context::CertContext;
    use schannel::cert_store::CertStore;

    let normalized = thumbprint
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_uppercase();
    if normalized.len() != 40 {
        return Err(format!(
            "Thumbprint must be 40 hex characters (got {}).",
            normalized.len()
        ));
    }
    let want = hex_decode(&normalized)
        .ok_or_else(|| "Thumbprint contains non-hex characters.".to_string())?;

    let store = CertStore::open_current_user("My")
        .map_err(|e| format!("Failed to open CurrentUser\\My: {e}"))?;

    let cert: CertContext = store
        .certs()
        .find(|c| c.sha1_thumbprint().map(|t| t.as_ref() == want.as_slice()).unwrap_or(false))
        .ok_or_else(|| format!("No certificate with thumbprint {normalized} found in CurrentUser\\My."))?;

    // Generate an ephemeral password and export to PFX.
    let password: String = (0..32)
        .map(|_| {
            let n: u32 = rand_u32() % 36;
            if n < 10 { (b'0' + n as u8) as char } else { (b'a' + (n - 10) as u8) as char }
        })
        .collect();

    let pfx = cert
        .to_pfx(&password)
        .map_err(|e| format!("Failed to export certificate to PFX: {e}"))?;

    Ok(ExportedPkcs12 { bytes: pfx, password })
}

#[cfg(not(windows))]
pub fn export_pkcs12_by_thumbprint(_thumbprint: &str) -> Result<ExportedPkcs12, String> {
    Err("Windows Certificate Store is only available on Windows.".into())
}

// ── helpers (Windows-only) ─────────────────────────────────────────────────

#[cfg(windows)]
fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 { return None; }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    for chunk in bytes.chunks(2) {
        let hi = (chunk[0] as char).to_digit(16)?;
        let lo = (chunk[1] as char).to_digit(16)?;
        out.push(((hi << 4) | lo) as u8);
    }
    Some(out)
}

#[cfg(windows)]
fn rand_u32() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Lightweight RNG — sufficient for an ephemeral export password.
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    nanos.wrapping_mul(2654435761)
}
