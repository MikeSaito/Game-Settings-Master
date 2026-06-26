use super::ScreenResolution;

fn parse_wh_output(text: &str) -> Option<ScreenResolution> {
    let (w, h) = text.split_once('x')?;
    let width: u32 = w.trim().parse().ok()?;
    let height: u32 = h.trim().parse().ok()?;
    if width > 0 && height > 0 && width <= 16384 && height <= 16384 {
        Some(ScreenResolution { width, height })
    } else {
        None
    }
}

#[test]
fn parses_resolution_string() {
    let r = parse_wh_output("2560x1440").unwrap();
    assert_eq!(r.width, 2560);
    assert_eq!(r.height, 1440);
}

#[test]
fn rejects_invalid_resolution() {
    assert!(parse_wh_output("0x1080").is_none());
    assert!(parse_wh_output("abc").is_none());
}
