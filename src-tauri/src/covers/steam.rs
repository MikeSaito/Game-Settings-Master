const STEAM_CDN: &str = "https://cdn.cloudflare.steamstatic.com/steam/apps";

pub fn steam_header_url(app_id: &str) -> String {
    format!("{STEAM_CDN}/{app_id}/header.jpg")
}
