use crate::{AdbTransports,  AdbTransportError, AdbCommand};
use anyhow::Result;
impl AdbTransports {
    async fn shell<S:ToString>(&mut self, serial: &Option<S>,cmd: String) -> Result<Vec<u8>> {
        let serial = match serial {
            Some(serial) => AdbCommand::TransportSerial(serial.to_string()),
            None => AdbCommand::TransportAny,
        };
        self.transports.send_command(serial, false).await?;
        self.transports.send_command(AdbCommand::ShellExec(cmd), true).await
      
    }
}
