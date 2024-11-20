use serde::{Deserialize, Serialize};
use reqwest::Error;
use chrono::{DateTime, Utc};
use serde_json::json;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Deserialize, Debug)]
struct IpResponse {
    ip: String,
}

#[derive(Deserialize, Debug)]
struct DnsRecord {
    id: String,
    zone_id: String,
    zone_name: String,
    name: String,
    #[serde(rename = "type")]
    type_field: String,
    content: String,
    proxiable: bool,
    proxied: bool,
    ttl: u32,
    comment: Option<String>,
    created_on: DateTime<Utc>,
    modified_on: DateTime<Utc>,
    comment_modified_on: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug)]
struct DnsResult {
    result: DnsRecord,
}

#[derive(Serialize, Debug)]
struct DnsPatchRequest {
    id: String,
    comment: Option<String>,
    name: String,
    proxied: bool,
    ttl: u32,
    content: String,
    #[serde(rename = "type")]
    type_field: String,
}

struct Configuration {
    zone_id: String,
    dns_record_id: String,
    bearer_token: String,
}

impl Configuration {
    pub fn load() -> Result<Configuration, Error> {
        let zone_id = env::var("ZONE_ID").expect("ZONE_ID must be set");
        let dns_record_id = env::var("DNS_RECORD_ID").expect("DNS_RECORD_ID must be set");
        let bearer_token = env::var("BEARER_TOKEN").expect("BEARER_TOKEN must be set");

        Ok(Configuration {
            zone_id,
            dns_record_id,
            bearer_token,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configuration = Configuration::load()?;
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut interval_timer = tokio::time::interval(chrono::Duration::seconds(30)
        .to_std().unwrap());

    while running.load(Ordering::SeqCst) {
        let dns_record = load_dns_records(&configuration).await?.result;
        'inner: loop {
            interval_timer.tick().await;
            println!("{:?}", dns_record);
            if do_stuff(&configuration, &dns_record).await? || !running.load(Ordering::SeqCst) {
                break 'inner;
            }
        }
    }
    Ok(())
}

async fn do_stuff(ref configuration: &Configuration, ref dns_record: &DnsRecord) -> Result<bool, Error> {
    let ip_response = load_external_ip().await?;
    let mut ip_updated = false;
    println!("{:?}", ip_response.ip);

    if ip_response.ip == dns_record.content {
        println!("Up to date!");
    } else {
        println!("IP needs to be updated");
        let patch_request = DnsPatchRequest {
            id: dns_record.id.to_string(),
            comment: Some("Ip updated by homelabdns".to_string()),
            name: dns_record.name.to_string(),
            proxied: dns_record.proxied,
            ttl: dns_record.ttl,
            content: dns_record.content.to_string(),
            type_field: dns_record.type_field.to_string(),
        };
        update_dns_record(&configuration, patch_request).await?;
        ip_updated = true;
    };
    Ok(ip_updated)
}

async fn load_external_ip() -> Result<IpResponse, Error> {
    let request_url = "https://api.ipify.org?format=json";
    let response = reqwest::get(request_url).await?;

    let response: IpResponse = response.json().await?;
    println!("{:?}", response.ip);
    Ok(response)
}

async fn load_dns_records(configuration: &Configuration) -> Result<DnsResult, Error> {
    let request_url = format!("https://api.cloudflare.com/client/v4/zones/{zoneId}/dns_records/{dnsRecordId}",
                              zoneId = configuration.zone_id, dnsRecordId = configuration.dns_record_id);

    println!("{}", request_url);

    let client = reqwest::Client::new();
    let response = client
        .get(request_url)
        .bearer_auth(&configuration.bearer_token)
        .send().await?;
    let dns_record: DnsResult = response.json().await?;

    Ok(dns_record)
}

async fn update_dns_record(configuration: &Configuration, dns_patch_request: DnsPatchRequest) -> Result<(), Error> {
    let request_url = format!("https://api.cloudflare.com/client/v4/zones/{zoneId}/dns_records/{dnsRecordId}",
                              zoneId = configuration.zone_id,
                              dnsRecordId = configuration.dns_record_id);

    println!("{}", request_url);
    let client = reqwest::Client::new();
    let response = client
        .patch(request_url)
        .json(&json!(dns_patch_request))
        .bearer_auth(&configuration.bearer_token)
        .send().await?;

    println!("{:?}", response.status());
    let dns_result: DnsResult = response.json().await?;

    println!("{:?}", dns_result);
    Ok(())
}