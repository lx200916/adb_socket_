use crate::result::dent::{SyncDent, SyncDentV1};
use crate::result::device::Device;
use crate::result::stat::StatInfo;
use crate::utils::check_path;
use crate::{result::device::Devices, AdbCommand, AdbTransportError, AdbTransports};
use anyhow::Result;
impl AdbTransports {
    #[async_backtrace::framed]
    pub async fn list<S: ToString>(
        &mut self,
        path: String,
        serial: Option<S>,
    ) -> Result<Vec<SyncDent>> {
        //check path
        let _ = check_path(path.clone())?;
        self.may_set_serial(serial).await?;
        self.transports
            .send_command(AdbCommand::Sync, false)
            .await?;

        self.sync_list_(path).await
    }
    #[async_backtrace::framed]
    async fn sync_list_(&mut self, path: String) -> Result<Vec<SyncDent>> {
        self.transports.send_sync_command(crate::AdbSyncModeCommand::List).await?;
        // println!("path: {}", path);
        let mut buf = Vec::new();
        let path_length = (path.len() as u32).to_le_bytes(); // Convert path_length to bytes
        buf.extend_from_slice(&path_length);
        buf.extend_from_slice(path.as_bytes());
        // println!("buf {:?}",buf);
        self.transports.write_all(&buf).await?;
        let mut dents: Vec<SyncDent> = Vec::new();
        loop {
            let msg = SyncDent::from_le_bytes_transport(self.transports.as_mut()).await;
            match msg {
                Ok(msg) => {
                    dents.push(msg);
                }
                Err(AdbTransportError::EOF)=> {break;}
                Err(err)=>{
                    return Err(err.into())
                }
            }
        }
        Ok(dents)
    }
}
