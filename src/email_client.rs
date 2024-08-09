//! src/email_client.rs
use reqwest::Client;

use crate::domain::SubscriberEmail;
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
}
impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
        }
    }
    // [...]
}
