use super::steam_header_url;

#[test]
fn steam_header_url_format() {
    assert_eq!(
        steam_header_url("1962700"),
        "https://cdn.cloudflare.steamstatic.com/steam/apps/1962700/header.jpg"
    );
}
