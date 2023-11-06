use crate::result::dent::{SyncDent, SyncDentV1};
use crate::result::device::Device;
use crate::result::stat::StatInfo;
use crate::utils::check_path;
use crate::{result::device::Devices, AdbCommand, AdbTransportError, AdbTransports};
use anyhow::Result;
impl AdbTransports {
    pub async fn list<S: ToString>(
        &mut self,
        path: String,
        serial: Option<S>,
    ) -> Result<Vec<SyncDent>> {
        //check path
        let pa = check_path(path.clone());
        if pa.is_err() || pa.unwrap() == false {
            return Err(anyhow::anyhow!("illegal path"));
        }
        let transport_ = match serial {
            Some(serial) => AdbCommand::TransportSerial(serial.to_string()),
            None => AdbCommand::TransportAny,
        };
        self.transports.send_command(transport_, false).await?;
        self.transports
            .send_command(AdbCommand::Sync, false)
            .await?;

        self.sync_list_(path).await
    }
    async fn sync_list_(&mut self, path: String) -> Result<Vec<SyncDent>> {
        let mut buf = Vec::new();
        let path_length = (path.len() as u32).to_le_bytes(); // Convert path_length to bytes
        buf.extend_from_slice(&path_length);
        buf.extend_from_slice(path.as_bytes());
        self.transports.write_all(&buf).await?;

        let mut msg = [0u8; std::mem::size_of::<SyncDentV1>()];
        let mut dents: Vec<SyncDent> = Vec::new();
        loop {
            self.transports
                .read_exact_(&mut msg)
                .await
                .map_err(|_| AdbTransportError::InvalidResponse)?;
            let msg = SyncDentV1::from_le_bytes(msg);
            match msg {
                Ok(msg) => {
                    if msg.namelen == 0 {
                        // return Err(AdbTransportError::InvalidResponse.into());
                        break;
                    } else {
                        let mut msg_name = vec![0u8; msg.namelen as usize];
                        self.transports
                            .read_exact_(&mut msg_name)
                            .await
                            .map_err(|_| AdbTransportError::InvalidResponse)?;
                        let name = String::from_utf8(msg_name)?;
                        let dent = msg.into_dent(name);
                        dents.push(dent);
                    }
                }

                Err(AdbTransportError::EOF) => {
                    break;
                }
                Err(e) => {
                    return Err(e.into());
                }
            };
        }
        Ok(dents)
    }
}
