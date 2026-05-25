pub const TEST_ENDPOINT:       &str = "https://tdl.integration.aws.fema.net/cap/SubmitCAPMessage";
pub const PRODUCTION_ENDPOINT: &str = "https://www.fema.gov/cap/COGProfile.do";

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct IpawsConfig {
    pub cog_id:            String,
    pub sender:            String,
    pub sender_name:       String,
    pub use_test_endpoint: bool,
    pub use_file_cert:     bool,
    pub cert_path:         String,
    #[serde(skip)]
    pub cert_password:     String,
    pub cert_thumbprint:   String,
    pub confirm_before_send: bool,
}

impl IpawsConfig {
    pub fn endpoint(&self) -> &str {
        if self.use_test_endpoint { TEST_ENDPOINT } else { PRODUCTION_ENDPOINT }
    }
    pub fn endpoint_label(&self) -> &str {
        if self.use_test_endpoint { "JITC Test Endpoint" } else { "Production Endpoint" }
    }
}
