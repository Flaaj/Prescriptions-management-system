use std::net::{IpAddr, Ipv4Addr};

use rocket::request::{FromRequest, Outcome};
use rocket_okapi::request::OpenApiFromRequest;

#[derive(Debug, PartialEq, Clone, OpenApiFromRequest)]
pub struct ClientRequestInfo {
    pub ip_address: IpAddr,
    pub user_agent: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientRequestInfo {
    type Error = ();

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let ip_address = req
            .client_ip()
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
        let user_agent = req
            .headers()
            .get_one("User-Agent")
            .unwrap_or("")
            .to_string();

        Outcome::Success(ClientRequestInfo {
            ip_address,
            user_agent,
        })
    }
}
