use crate::printer::print_data;
use failure::{format_err, Error};
use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::*;
use std::fmt;

#[derive(Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum SessionType {
    Insecure = 0,
    Secure = 1,
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SessionType::Insecure => write!(f, "Insecure"),
            SessionType::Secure => write!(f, "Secure"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlexSession {
    pub container: String,
    pub full_title: String,
    pub ip_address_public: String,
    pub media_type: String,
    pub platform: String,
    pub player: String,
    pub quality_profile: String,
    pub state: String,
    pub transcode_container: String,
    pub user: String,
    #[serde(rename = "secure")]
    // this was only added recently to the API so its optional
    pub session_type: Option<SessionType>,
    pub progress_percent: String, // wish this were some sort of Int
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlexpyData {
    pub sessions: Vec<PlexSession>,
    pub stream_count: String,
    pub total_bandwidth: i64,
    pub stream_count_transcode: i64,
    pub wan_bandwidth: i64,
    pub stream_count_direct_play: i64,
    pub lan_bandwidth: i64,
    pub stream_count_direct_stream: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlexpyResponse {
    #[serde(deserialize_with = "optional_data")]
    pub data: Option<PlexpyData>,
    #[serde(deserialize_with = "null_string")]
    pub message: String,
    pub result: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInfo {
    pub response: PlexpyResponse,
}

/// Used to translate a `null` from the Plexpy API as an empty string.
fn null_string<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(Option::unwrap_or_default)
}

/// The Plexpy API is less than ideal in that it will return a success even when the user passes
/// something like an invalid api key.  So this allows us to return Some(PlexpyData) or None which
/// lets us fully parse the json object returned containing the real error message in
/// the "message" field.
///
/// Example:
/// ```
/// {
///  "response": {
///    "message": "Invalid apikey",
///    "data": {},
///    "result": "success"
///  }
///}
/// ```
fn optional_data<'de, D>(d: D) -> Result<Option<PlexpyData>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Deserialize::deserialize(d).ok())
}

/// Request the Plex server's current activity from a plexpy server
pub fn get_activity<T: AsRef<str>>(server: T, key: T) -> Result<(), Error> {
    let mut server = Url::parse(server.as_ref())?;
    server.set_path("/api/v2");
    server
        .query_pairs_mut()
        .append_pair("apikey", key.as_ref())
        .append_pair("cmd", "get_activity");

    let plex: ServerInfo = reqwest::get(server)?.json()?;

    // the API gave us a 200 response but the "message" field contains an error
    if plex.response.message != "" && plex.response.data.is_none() {
        return Err(format_err!("{}", &plex.response.message));
    }

    if let Some(data) = plex.response.data {
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        print_data(&mut stdout_lock, &data)?;
    }

    Ok(())
}
