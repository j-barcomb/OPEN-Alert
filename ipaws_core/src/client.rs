use crate::{channels::IpawsConfig, models::CapAlert, xml};
use std::time::{Duration, Instant};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Certificate error: {0}")]
    Certificate(String),
    #[error("HTTP {status}: {message}")]
    HttpStatus { status: u16, message: String },
    #[error("Transport error: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct IpawsResponse {
    pub is_success:        bool,
    pub alert_identifier:  Option<String>,
    pub server_message_id: Option<String>,
    pub errors:            Vec<String>,
    pub elapsed:           Duration,
    pub submitted_at:      chrono::DateTime<chrono::Utc>,
    pub raw_body:          String,
}

pub struct IpawsClient {
    config: IpawsConfig,
}

impl IpawsClient {
    pub fn new(config: IpawsConfig) -> Self {
        Self { config }
    }

    fn build_client(&self) -> Result<reqwest::blocking::Client, ClientError> {
        let mut builder = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30));

        if self.config.use_file_cert {
            if self.config.cert_path.is_empty() {
                return Err(ClientError::Certificate(
                    "Certificate path is not configured.".into(),
                ));
            }
            let bytes = std::fs::read(&self.config.cert_path)?;
            let identity = reqwest::Identity::from_pkcs12_der(
                &bytes,
                &self.config.cert_password,
            )
            .map_err(|e| ClientError::Certificate(e.to_string()))?;
            builder = builder.identity(identity);
        } else {
            // Windows Certificate Store loading is platform-specific.
            #[cfg(windows)]
            {
                use crate::winstore;
                if self.config.cert_thumbprint.is_empty() {
                    return Err(ClientError::Certificate(
                        "Certificate thumbprint is not configured.".into(),
                    ));
                }
                let pkcs12 = winstore::export_pkcs12_by_thumbprint(
                    &self.config.cert_thumbprint,
                )
                .map_err(ClientError::Certificate)?;
                let identity = reqwest::Identity::from_pkcs12_der(&pkcs12.bytes, &pkcs12.password)
                    .map_err(|e| ClientError::Certificate(e.to_string()))?;
                builder = builder.identity(identity);
            }
            #[cfg(not(windows))]
            {
                return Err(ClientError::Certificate(
                    "Windows Certificate Store lookup is only available on Windows. \
                     Use file-based certificate on this platform.".into(),
                ));
            }
        }

        builder.build().map_err(ClientError::Transport)
    }

    /// Submit a CAP alert to IPAWS-OPEN. Blocks the calling thread.
    pub fn submit(&self, alert: &CapAlert) -> IpawsResponse {
        let start = Instant::now();
        let submitted_at = chrono::Utc::now();
        let alert_id = alert.identifier.clone();
        let xml_body = xml::serialize(alert, false);

        match self.submit_inner(&xml_body) {
            Ok((server_id, body)) => IpawsResponse {
                is_success:        true,
                alert_identifier:  Some(alert_id),
                server_message_id: Some(server_id),
                errors:            Vec::new(),
                elapsed:           start.elapsed(),
                submitted_at,
                raw_body:          body,
            },
            Err(e) => IpawsResponse {
                is_success:        false,
                alert_identifier:  Some(alert_id),
                server_message_id: None,
                errors:            vec![e.to_string()],
                elapsed:           start.elapsed(),
                submitted_at,
                raw_body:          String::new(),
            },
        }
    }

    fn submit_inner(&self, xml_body: &str) -> Result<(String, String), ClientError> {
        let client = self.build_client()?;
        let url = self.config.endpoint();

        let response = client
            .post(url)
            .header("Content-Type", "application/xml; charset=UTF-8")
            .body(xml_body.to_string())
            .send()?;

        let status = response.status();
        let body = response.text().unwrap_or_default();

        if status.is_success() {
            let server_id = extract_server_id(&body).unwrap_or_else(|| "—".into());
            Ok((server_id, body))
        } else {
            let messages = parse_error_body(&body, status.as_u16());
            Err(ClientError::HttpStatus {
                status:  status.as_u16(),
                message: messages.join("; "),
            })
        }
    }
}

/// Extract the IPAWS-assigned server-side message ID from a response body.
/// Looks for `<identifier>` or `<msgId>` elements (case-sensitive, CAP convention).
pub(crate) fn extract_server_id(body: &str) -> Option<String> {
    for tag_open in &["<identifier>", "<msgId>"] {
        let tag_close = tag_open.replace('<', "</");
        if let Some(start) = body.find(tag_open) {
            let rest = &body[start + tag_open.len()..];
            if let Some(end) = rest.find(&tag_close) {
                let v = rest[..end].trim();
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

/// Parse error messages from an IPAWS XML error response body.
/// Falls back to a generic message if no `<message>` elements are found.
pub(crate) fn parse_error_body(body: &str, status: u16) -> Vec<String> {
    if body.trim().is_empty() {
        return vec![format!("HTTP {status} with empty body.")];
    }
    let mut messages = Vec::new();
    let mut search = body;
    while let Some(start) = search.find("<message>") {
        let rest = &search[start + "<message>".len()..];
        match rest.find("</message>") {
            Some(end) => {
                let msg = rest[..end].trim();
                if !msg.is_empty() {
                    messages.push(msg.to_string());
                }
                search = &rest[end + "</message>".len()..];
            }
            None => break,
        }
    }
    if messages.is_empty() {
        let excerpt = &body[..body.len().min(300)];
        messages.push(format!("HTTP {status}: {}", excerpt.trim()));
    }
    messages
}

// ── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{extract_server_id, parse_error_body};

    #[test]
    fn extract_server_id_finds_identifier_tag() {
        let body = r#"<?xml version="1.0"?><response><identifier>SRV-9876</identifier></response>"#;
        assert_eq!(extract_server_id(body), Some("SRV-9876".to_string()));
    }

    #[test]
    fn extract_server_id_falls_back_to_msgid() {
        let body = r#"<response><msgId>ID-42</msgId></response>"#;
        assert_eq!(extract_server_id(body), Some("ID-42".to_string()));
    }

    #[test]
    fn extract_server_id_returns_none_when_absent() {
        let body = r#"<response>ok</response>"#;
        assert_eq!(extract_server_id(body), None);
    }

    #[test]
    fn extract_server_id_trims_whitespace() {
        let body = "<identifier>\n   SRV-1\n  </identifier>";
        assert_eq!(extract_server_id(body), Some("SRV-1".to_string()));
    }

    #[test]
    fn extract_server_id_ignores_empty_value() {
        let body = "<identifier></identifier>";
        assert_eq!(extract_server_id(body), None);
    }

    #[test]
    fn parse_error_body_extracts_message_tags() {
        let body = "<error><message>SAME code missing</message><message>Bad scope</message></error>";
        let msgs = parse_error_body(body, 400);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0], "SAME code missing");
        assert_eq!(msgs[1], "Bad scope");
    }

    #[test]
    fn parse_error_body_handles_empty_body() {
        let msgs = parse_error_body("", 500);
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("500"));
        assert!(msgs[0].contains("empty body"));
    }

    #[test]
    fn parse_error_body_falls_back_to_excerpt() {
        let body = "Internal Server Error: database connection refused";
        let msgs = parse_error_body(body, 500);
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("500"));
        assert!(msgs[0].contains("Internal Server Error"));
    }

    #[test]
    fn parse_error_body_truncates_long_excerpts() {
        let body = "X".repeat(1000);
        let msgs = parse_error_body(&body, 502);
        // Should not blow up; excerpt is bounded.
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].len() < 400);
    }
}
