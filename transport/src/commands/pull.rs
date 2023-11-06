use std::path::PathBuf;

use crate::{AdbCommand, AdbTransportError, AdbTransports};
use anyhow::Result;
const SYNC_DATA_MAX: usize = 64 * 1024;
const ID_DONE: u32 = u32::from_be_bytes([b'D', b'O', b'N', b'E']);
const ID_DATA: u32 = u32::from_be_bytes([b'D', b'A', b'T', b'A']);
impl AdbTransports {
    pub async fn pull<S: ToString, A: AsRef<str>>(
        &mut self,
        serial: Option<S>,
        path: A,
        output: &mut dyn std::io::Write,
    ) -> Result<()> {
        let serial = match serial {
            Some(serial) => AdbCommand::TransportSerial(serial.to_string()),
            None => AdbCommand::TransportAny,
        };
        self.transports.send_command(serial, false).await?;
        self.transports
            .send_command(AdbCommand::Sync, false)
            .await?;
        self.sync_recv(path.as_ref().to_string(), output).await?;
        Ok(())
    }
    async fn sync_recv(&mut self, path: String, output: &mut dyn std::io::Write) -> Result<()> {
        if path.len() > 1024 {
            return Err(AdbTransportError::AdbError("Too long Path".into()).into());
        }
        if path.contains("\n") {
            return Err(AdbTransportError::AdbError("Path contains newline".into()).into());
        }
        if path.is_empty() {
            return Err(AdbTransportError::AdbError("Path is empty".into()).into());
        }

        //TODO: Recv-2 Feature.
        self.transports
            .send_sync_command(crate::AdbSyncModeCommand::Recv)
            .await?;
        let mut buf = Vec::new();
        let path_length = (path.len() as u32).to_le_bytes(); // Convert path_length to bytes
        buf.extend_from_slice(&path_length);
        buf.extend_from_slice(path.as_bytes());
        self.transports.write_all(&buf).await?;
        // https://cs.android.com/android/platform/superproject/main/+/main:packages/modules/adb/client/file_sync_client.cpp;l=1098;drc=60c3258770b1ce3ce5bbdcff3c4a87c8f996b92f;bpv=1;bpt=1
        let mut bytes_copied = 0;
        let mut buffer = vec![0; SYNC_DATA_MAX as usize];
        let mut sync_msg_data = [0_u8; 8];
        loop {
            self.transports
                .read_exact_(&mut sync_msg_data)
                .await
                .map_err(|err| AdbTransportError::IoError(err))?;
            let (id, size) = sync_msg_data.split_at(4);
            let id = u32::from_be_bytes(id.try_into().map_err(|err| {
                AdbTransportError::AdbError(format!("Invalid Sync Message ID: {}", err))
            })?);
            let size = u32::from_be_bytes(size.try_into().map_err(|err| {
                AdbTransportError::AdbError(format!("Invalid Sync Message Size: {}", err))
            })?);
            if id == ID_DONE {
                break;
            }
            if id != ID_DATA {
                return Err(AdbTransportError::AdbError(format!(
                    "Invalid Sync Message ID: {}",
                    id
                ))
                .into());
            }
            buffer.resize(size as usize, 0);
            self.transports
                .read_exact_(&mut buffer)
                .await
                .map_err(|err| AdbTransportError::IoError(err))?;
            output
                .write_all(&buffer)
                .map_err(|err| AdbTransportError::IoError(err))?;
            bytes_copied += size;
        }
        Ok(())
    }
}
