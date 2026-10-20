//! Verify the exact updater bytes and manifest before publishing.
use base64::Engine;
use minisign_verify::{PublicKey, Signature};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 {
        return Err("usage: verify_release INSTALLER SIGNATURE MANIFEST REPOSITORY".into());
    }
    let config: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json"))?;
    let manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(&args[3])?)?;
    let signature = std::fs::read_to_string(&args[2])?.trim().to_owned();
    let decode = |s: &str| -> Result<String, Box<dyn std::error::Error>> {
        Ok(String::from_utf8(
            base64::engine::general_purpose::STANDARD.decode(s)?,
        )?)
    };
    let key = PublicKey::decode(&decode(
        config["plugins"]["updater"]["pubkey"]
            .as_str()
            .ok_or("Missing updater key")?,
    )?)?;
    let sig = Signature::decode(&decode(&signature)?)?;
    let bytes = std::fs::read(&args[1])?;
    key.verify(&bytes, &sig, false)?;
    let version = config["version"].as_str().ok_or("Missing version")?;
    let name = std::path::Path::new(&args[1])
        .file_name()
        .unwrap()
        .to_str()
        .ok_or("Invalid filename")?;
    let platform = &manifest["platforms"]["windows-x86_64"];
    let url = format!(
        "https://github.com/{}/releases/download/v{version}/{name}",
        args[4]
    );
    if manifest["version"] != version
        || platform["signature"] != signature
        || platform["url"] != url
    {
        return Err("Updater manifest does not match the verified installer".into());
    }
    println!("Installer signature and update manifest verified");
    Ok(())
}
