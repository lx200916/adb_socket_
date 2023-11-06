use crate::{AdbTransports,  AdbTransportError, AdbCommand};
use anyhow::Result;
impl AdbTransports {
    pub async fn shell<S:ToString>(&mut self, serial: &Option<S>,cmd: String) -> Result<Vec<u8>> {
        let serial = match serial {
            Some(serial) => AdbCommand::TransportSerial(serial.to_string()),
            None => AdbCommand::TransportAny,
        };
        self.transports.send_command(serial, false).await?;
        self.transports.send_command(AdbCommand::ShellExec(cmd), true).await
      
    }
    pub async fn mkdir<S:ToString,A:AsRef<str>>(&mut self, serial: Option<S>,path: A) -> Result<Vec<u8>> {
        let path = path.as_ref();
        if path.len() > 1024 {
            return Err(AdbTransportError::AdbError("Too long Path".into()).into());
        }
        if path.is_empty() {
            return Err(AdbTransportError::AdbError("Path is empty".into()).into());
        }
        if path=="/" {
            return Err(AdbTransportError::AdbError("Path illegal".into()).into());
        }
        let serial = match serial {
            Some(serial) => AdbCommand::TransportSerial(serial.to_string()),
            None => AdbCommand::TransportAny,
        };
        self.transports.send_command(serial, false).await?;
        let cmd = format!("mkdir {}",path);
        self.transports.send_command(AdbCommand::ShellExec(cmd), true).await
    }
}
