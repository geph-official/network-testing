use std::io::Read;
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

#[derive(Debug, Deserialize)]
pub struct Monitor {
    id: String,
    #[serde(rename = "type")]
    type_field: String,
    pub attributes: MonitorAttributes,
}

#[derive(Debug, Deserialize)]
pub struct MonitorAttributes {
    pub url: String,
    pub port: Option<String>,
    pub status: String,
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
    println!("body: {:?}", body);

    // Filter the array for monitors with a status of "up"
    let active_monitors: Vec<Monitor> = body
        .data
        .into_iter()
        .filter(|monitor| monitor.attributes.status == "up")
        .collect();

    Ok(active_monitors)
}

pub async fn execute_script(host: &str, port: &str, script_url: &str) -> anyhow::Result<()> {
    let session = connect_to_host(host, port.parse()?)?;
    download_and_run_script(&session, script_url)?;
    Ok(())
}

fn download_and_run_script(session: &ssh2::Session, script_url: &str) -> anyhow::Result<String> {
    let script_file_name = script_url
        .split('/')
        .last()
        .expect("script URL is empty")
        .to_string();
    println!("SCRIPT: {script_file_name}");

    // Create a new channel for the curl command
    let mut channel = session.channel_session()?;
    channel.exec(&format!("curl -O {}", script_url))?;
    read_all_channel_output(&mut channel)?;
    let status = channel.exit_status()?;

    println!("curl status: {}", status);

    if status != 0 {
        return Err(anyhow::anyhow!("Failed to download script"));
    }

    // Create a new channel for the chmod command
    let mut channel = session.channel_session()?;
    channel.exec(&format!("chmod +x {}", script_file_name))?;
    read_all_channel_output(&mut channel)?;
    let status = channel.exit_status()?;

    if status != 0 {
        return Err(anyhow::anyhow!("Failed to change script permissions"));
    }

    // Create a new channel for the script execution
    let mut channel = session.channel_session()?;
    channel.exec(&script_file_name)?;
    read_all_channel_output(&mut channel)?;
    let status = channel.exit_status()?;

    if status == 0 {
        Ok("successfully ran script".into())
    } else {
        Err(anyhow::anyhow!("Failed to run script"))
    }
}

fn read_all_channel_output(channel: &mut ssh2::Channel) -> anyhow::Result<()> {
    let mut output = Vec::new();
    channel.read_to_end(&mut output)?;
    channel.wait_close()?;
    Ok(())
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
