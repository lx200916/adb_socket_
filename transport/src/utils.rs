use anyhow::{Result};

use crate::{transport::transport::AdbTransport, AdbTransportError};
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
        return Ok(false);
    }
Ok(true)
}

#[inline]
pub async fn get_fail_message(transport:&mut dyn AdbTransport)->Result<String,AdbTransportError>{
                    let length = transport.get_length().await.map_err(|err|AdbTransportError::InvalidResponse(String::from("get_fail_message"), Some(err.to_string())))?;
                let mut message = vec![0u8; length];
                transport.read_exact_(&mut message).await?;
                let message = std::str::from_utf8(&message).map_err(|err|AdbTransportError::InvalidResponse(String::from("get_fail_message"), Some(err.to_string())))?;
                Ok(message.to_string())
}