use anyhow::Result;
pub fn check_path<S:ToString>(path:S)->Result<bool>{
    let path = path.to_string();
    if path.len() > 1024 {
        return Err(anyhow::anyhow!("Path too long"));
    }
    if path.is_empty() {
        return Err(anyhow::anyhow!("Path is empty"));
    }
    if path.contains('\0') {
        return Err(anyhow::anyhow!("Path contains null byte"));
    }
    if path.contains("//") {
        return Err(anyhow::anyhow!("Path contains double slash"));
    }
    if path == "/" {
        return Err(anyhow::anyhow!("Path is root"));
    }
    if path.starts_with("/") && !path.starts_with("/data/local/tmp"){
        return Err(anyhow::anyhow!("illegal path"));
    }
Ok(true)
}