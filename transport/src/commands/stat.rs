use crate::result::device::Device;
use crate::result::stat::StatInfo;
use crate::{result::device::Devices, AdbCommand, AdbTransportError, AdbTransports};
use anyhow::Result;

impl AdbTransports {
    #[async_backtrace::framed]
    pub async fn sync_stat<S: ToString>(
        &mut self,
        path: String,
        serial: Option<S>,
    ) -> Result<StatInfo> {
        
        let transport_ = match serial {
            Some(serial) => AdbCommand::TransportSerial(serial.to_string()),
            None => AdbCommand::TransportAny,
        };
        self.transports.send_command(transport_, false).await?;
        self.transports
            .send_command(AdbCommand::Sync, false)
            .await?;

        self.sync_stat_(path).await
    }
    #[async_backtrace::framed]
    async fn sync_stat_(&mut self, path: String) -> Result<StatInfo> {
        self.transports
            .send_sync_command(crate::AdbSyncModeCommand::Stat)
            .await?;
        let mut buf = Vec::new();
        let path_length = (path.len() as u32).to_le_bytes(); // Convert path_length to bytes
        buf.extend_from_slice(&path_length);
        buf.extend_from_slice(path.as_bytes());
        self.transports.write_all(&buf).await?;
        let msg = StatInfo::from_le_bytes_transport(self.transports.as_mut()).await?;
        Ok(msg)
    }
}

