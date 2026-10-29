use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub ip: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub loc: String,
    pub org: String,
    pub timezone: String,
    pub colo: Option<String>,
}

impl Default for NetworkInfo {
    fn default() -> Self {
        Self {
            ip: "Unknown".to_string(),
            city: "Unknown".to_string(),
            region: "Unknown".to_string(),
            country: "Unknown".to_string(),
            loc: "Unknown".to_string(),
            org: "Unknown".to_string(),
            timezone: "Unknown".to_string(),
            colo: None,
        }
    }
}

fn extract_colo(headers: &reqwest::header::HeaderMap) -> Option<String> {
    // 1. Try direct "colo" header (e.g. "colo: DAC")
    if let Some(colo) = headers.get("colo") {
        if let Ok(s) = colo.to_str() {
            if !s.trim().is_empty() {
                return Some(s.trim().to_string());
            }
        }
    }
    // 2. Try "cf-meta-colo" header
    if let Some(colo) = headers.get("cf-meta-colo") {
        if let Ok(s) = colo.to_str() {
            if !s.trim().is_empty() {
                return Some(s.trim().to_string());
            }
        }
    }
    // 3. Fallback: Parse from CF-RAY (e.g. "a49b32fef81dadfb-DAC" -> "DAC")
    if let Some(ray) = headers.get("cf-ray") {
        if let Ok(ray_str) = ray.to_str() {
            if let Some((_, pop)) = ray_str.split_once('-') {
                let trimmed = pop.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

pub async fn fetch_network_info(client: &reqwest::Client) -> Result<NetworkInfo, anyhow::Error> {
    // 1. Query Cloudflare endpoint to get accurate Edge PoP colo and geo headers
    let cf_res = client
        .get("https://speed.cloudflare.com/__down?bytes=0")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    let mut info = NetworkInfo::default();

    if let Ok(resp) = cf_res {
        info.colo = extract_colo(resp.headers());
        if let Some(ip) = resp.headers().get("cf-meta-ip") {
            if let Ok(s) = ip.to_str() {
                info.ip = s.to_string();
            }
        }
        if let Some(asn) = resp.headers().get("cf-meta-asn") {
            if let Ok(s) = asn.to_str() {
                info.org = format!("AS{}", s);
            }
        }
        if let Some(country) = resp.headers().get("cf-meta-country") {
            if let Ok(s) = country.to_str() {
                info.country = s.to_string();
            }
        }
        if let Some(city) = resp.headers().get("cf-meta-city") {
            if let Ok(s) = city.to_str() {
                info.city = s.to_string();
            }
        }
    }

    // 2. Supplement details from ipinfo for richer region/org/loc descriptions
    let ipinfo_res = client
        .get("https://ipinfo.io/json")
        .timeout(std::time::Duration::from_secs(4))
        .send()
        .await;

    if let Ok(resp) = ipinfo_res {
        if resp.status().is_success() {
            if let Ok(ip_data) = resp.json::<NetworkInfo>().await {
                if !ip_data.ip.is_empty() && info.ip == "Unknown" {
                    info.ip = ip_data.ip;
                }
                if !ip_data.org.is_empty() {
                    info.org = ip_data.org;
                }
                if !ip_data.city.is_empty() {
                    info.city = ip_data.city;
                }
                if !ip_data.region.is_empty() {
                    info.region = ip_data.region;
                }
                if !ip_data.country.is_empty() {
                    info.country = ip_data.country;
                }
                if !ip_data.loc.is_empty() {
                    info.loc = ip_data.loc;
                }
                if !ip_data.timezone.is_empty() {
                    info.timezone = ip_data.timezone;
                }
            }
        }
    }

    Ok(info)
}
