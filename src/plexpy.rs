use crate::printer::print_data;
use anyhow::{anyhow, Error};
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
pub struct PlexpyHistoryData {
    #[serde(rename = "recordsTotal")]
    pub records_total: i64,
    #[serde(rename = "data")]
    pub history: Vec<HistoryEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HistoryEntry {
    pub full_title: String,
    pub player: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlexpyActivityData {
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
#[serde(untagged)]
pub enum PlexpyData {
    History(PlexpyHistoryData),
    Activity(PlexpyActivityData),
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
fn optional_data<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Deserialize::deserialize(d).ok())
}

enum RequestCmd {
    GetActivity,
    GetHistory,
}

impl RequestCmd {
    fn as_str(&self) -> &'static str {
        match self {
            RequestCmd::GetActivity => "get_activity",
            RequestCmd::GetHistory => "get_history",
        }
    }
}

fn do_request(url: Url) -> Result<(), Error> {
    let server: ServerInfo = reqwest::blocking::get(url)?.json()?;
    // the API gave us a 200 response but the "message" field contains an error
    if server.response.message != "" && server.response.data.is_none() {
        return Err(anyhow!("{}", &server.response.message));
    }

    if let Some(data) = server.response.data {
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        print_data(&mut stdout_lock, &data)?;
    }
    Ok(())
}

fn prepare_url(server: &str, key: &str, cmd: RequestCmd) -> Result<Url, Error> {
    let mut url = Url::parse(server)?;
    url.set_path("/api/v2");
    url.query_pairs_mut()
        .append_pair("apikey", key)
        .append_pair("cmd", cmd.as_str());
    Ok(url)
}

/// Request the Plex server's current activity from a plexpy server
pub fn get_activity<T: AsRef<str>>(server: T, key: T) -> Result<(), Error> {
    let url = prepare_url(server.as_ref(), key.as_ref(), RequestCmd::GetActivity)?;
    do_request(url)
}

/// Request the Plex server's history from a plexpy server
pub fn get_history<T: AsRef<str>>(server: T, key: T, entries: &str) -> Result<(), Error> {
    let mut url = prepare_url(server.as_ref(), key.as_ref(), RequestCmd::GetHistory)?;
    url.query_pairs_mut().append_pair("length", entries);
    do_request(url)
}
