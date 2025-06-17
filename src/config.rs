#[derive(serde::Serialize)]
pub(crate) struct Config {
    pub(crate) ntp_server: String,
    pub(crate) ip_check: String,
    pub(crate) url_check: String,
}
