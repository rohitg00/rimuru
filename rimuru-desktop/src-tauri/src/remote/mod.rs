pub mod server;

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct RemoteStatus {
    pub running: bool,
    pub url: Option<String>,
    pub qr_svg: Option<String>,
}
