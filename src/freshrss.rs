use anyhow::{bail, Result};
use embedded_svc::http::Method::Get;
use embedded_svc::{http::client::Client, io::Read};
use esp_idf_svc::http::client::EspHttpConnection;
use log::info;

pub fn freshrss_connect(
    cli: &mut Client<EspHttpConnection>,
    domain: &str,
    username: &str,
    password: &str,
) -> Result<String> {
    let request_url = [
        "https://",
        domain,
        "/api/greader.php/accounts/ClientLogin?Email=",
        username,
        "&Passwd=",
        password,
    ]
    .join("");
    let request = cli.get(request_url.as_str())?;
    let response = request.submit()?;

    let status = response.status();
    info!("Response code : {:?}", status);
    info!("Response message : {:?}", response.status_message());

    let auth_string;
    match status {
        200..=299 => {
            let mut buf = [0_u8; 256];
            let mut reader = response;
            let mut response_text = String::new();
            loop {
                if let Ok(size) = Read::read(&mut reader, &mut buf) {
                    if size == 0 {
                        break;
                    }
                    response_text.push_str(std::str::from_utf8(&buf[..size])?);
                    info!("{}", response_text);
                }
            }
            auth_string = [
                "GoogleLogin auth=",
                response_text
                    .rsplit("=")
                    .next()
                    .expect("Couldn't find auth string")
                    .trim_end_matches("\n"),
            ]
            .join("");
        }
        _ => bail!("Unexpected response code: {}", status),
    };
    info!("auth : {:?}", auth_string);

    Ok(auth_string)
}

pub fn freshrss_get_articles(
    cli: &mut Client<EspHttpConnection>,
    auth_string: &str,
    domain: &str,
) -> Result<String> {
    let request_url = [
        "https://",
        domain,
        "/api/greader.php/reader/api/0/stream/contents/reading-list?output=json&n=5",
    ]
    .join("");
    let auth_header = [("Authorization", auth_string)];
    let request = cli.request(Get, &request_url, &auth_header)?;
    let response = request.submit()?;

    let status = response.status();
    info!("Response code : {:?}", status);
    info!("Response message : {:?}", response.status_message());

    let mut response_text = String::new();
    match status {
        200..=299 => {
            let mut buf = [0_u8; 1024];
            let mut reader = response;
            loop {
                if let Ok(size) = Read::read(&mut reader, &mut buf) {
                    if size == 0 {
                        break;
                    }
                    response_text.push_str(std::str::from_utf8(&buf[..size])?);
                }
            }
            info!("Response text : {:?}", response_text);
        }
        _ => bail!("Unexpected response code: {}", status),
    };
    Ok(response_text)
}
