use std::net::TcpStream;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use serde::Deserialize;

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct RequestHeader {
    id: String,
    name: String,
    value: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct MonitorAttributes {
    pub url: String,
    pronounceable_name: String,
    monitor_type: String,
    monitor_group_id: String,
    last_checked_at: String,
    status: String,
    required_keyword: String,
    verify_ssl: bool,
    check_frequency: i32,
    call: bool,
    sms: bool,
    email: bool,
    push: bool,
    team_wait: Option<String>,
    http_method: String,
    request_timeout: i32,
    recovery_period: i32,
    request_headers: Vec<RequestHeader>,
    request_body: String,
    paused_at: Option<String>,
    created_at: String,
    updated_at: String,
    ssl_expiration: i32,
    domain_expiration: i32,
    regions: Vec<String>,
    pub port: Option<i32>,
    confirmation_period: i32,
    expected_status_codes: Vec<String>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Monitor {
    id: String,
    #[serde(rename(deserialize = "type"))]
    type_field: String,
    pub attributes: MonitorAttributes,
}

#[derive(Debug, Deserialize)]
struct ResponseData {
    data: Vec<Monitor>,
}

pub async fn list_active_monitors() -> anyhow::Result<Vec<Monitor>> {
    let team_token = std::env::var("UPTIME_API_KEY")?;
    // Create a new HeaderMap and add the authorization header
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", team_token))?,
    );

    // Create a new client
    let client = Client::new();

    // Make the GET request
    let res = client
        .get("https://uptime.betterstack.com/api/v2/monitors")
        .headers(headers)
        .send()
        .await?;

    // Parse the JSON body
    let body: ResponseData = res.json().await?;

    // Filter the array for monitors with a status of "up"
    let active_monitors: Vec<Monitor> = body
        .data
        .into_iter()
        .filter(|monitor| monitor.attributes.status == "up")
        .collect();

    Ok(active_monitors)
}

pub async fn execute_script(host: &str, port: i32, script_url: &str) -> anyhow::Result<()> {
    let session = connect_to_host(host, port)?;
    download_and_run_script(&session, script_url)?;
    Ok(())
}

fn download_and_run_script(session: &ssh2::Session, script_url: &str) -> anyhow::Result<String> {
    let mut channel = session.channel_session()?;
    let script_file_name = format!("$(basename {})", script_url); // Extract file name from URL
    channel.exec(&format!("curl -O {}", script_url))?;
    channel.wait_close()?;
    let status = channel.exit_status()?;
    if status == 0 {
        // make it executable
        let mut channel = session.channel_session()?;
        channel.exec(&format!("chmod +x {}", script_file_name))?;
        channel.exec(&script_file_name)?;
        channel.wait_close()?;
        Ok("successfully ran script".into())
    } else {
        anyhow::bail!("Failed to download script");
    }
}

fn connect_to_host(host: &str, port: i32) -> anyhow::Result<ssh2::Session> {
    // TCP connection
    let tcp = TcpStream::connect(format!("{}:{}", host, port))?;
    let mut session = ssh2::Session::new().unwrap();
    session.set_tcp_stream(tcp);
    session.handshake()?;

    // Use the username "root" for authentication
    session.userauth_agent("root")?;

    // Make sure we succeeded
    if session.authenticated() {
        Ok(session)
    } else {
        anyhow::bail!("Authentication failed");
    }
}
