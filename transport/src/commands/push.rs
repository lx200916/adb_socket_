use std::{io::Read, path::PathBuf};

use crate::{AdbCommand, AdbRespStatus, AdbTransportError, AdbTransports};
use anyhow::Result;
const SYNC_DATA_MAX: usize = 64 * 1024;
const ID_DONE: u32 = u32::from_be_bytes([b'D', b'O', b'N', b'E']);
const ID_DATA: u32 = u32::from_be_bytes([b'D', b'A', b'T', b'A']);
impl AdbTransports {
    #[async_backtrace::framed]
    pub async fn push<S: ToString, A: AsRef<str>>(
        &mut self,
        serial: Option<S>,
        stream: &mut dyn Read,
        path: A,
    ) -> Result<()> {
        self.new_connection().await?;
        self.may_set_serial(serial).await?;


        self.transports
            .send_command(AdbCommand::Sync, false)
            .await?;
        self.sync_send(stream, path.as_ref().to_string()).await?;
        Ok(())
    }
    async fn sync_send(&mut self, input_stream: &mut dyn Read, path: String) -> Result<()> {
        //TODO: change premision
        let path_with_premission = format!("{},{}", path, "0644");
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
            .send_sync_command(crate::AdbSyncModeCommand::Send)
            .await?;
        let mut buf = Vec::new();
        let path_length = (path_with_premission.len() as u32).to_le_bytes(); // Convert path_length to bytes
        buf.extend_from_slice(&path_length);
        buf.extend_from_slice(path_with_premission.as_bytes());
        self.transports.write_all(&buf).await?;

        let mut buffer = vec![0; SYNC_DATA_MAX as usize];
        loop {
            let bytes_read = input_stream.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            let mut chunk_len_buf = [0_u8; 4];
            chunk_len_buf.copy_from_slice(&(bytes_read as u32).to_le_bytes());
            self.transports.write_all(b"DATA").await?;
            self.transports.write_all(&chunk_len_buf).await?;
            self.transports.write_all(&buffer[..bytes_read]).await?;
        }
        self.transports.write_all(b"DONE").await?;
        let last_modified =
            match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
                Ok(n) => n,
                Err(_) => panic!("SystemTime before UNIX EPOCH!"),
            };
        let last_modified_buf = (last_modified.as_secs() as u32).to_le_bytes();
        self.transports.write_all(&last_modified_buf).await?;
        let mut request_status = [0; 4];
        self.transports.read_exact_(&mut request_status).await?;

        match AdbRespStatus::from(request_status) {
            AdbRespStatus::Okay => Ok(()),
            AdbRespStatus::Fail(_) => {
                let length = self
                    .transports
                    .get_length()
                    .await
                    .map_err(|err| AdbTransportError::InvalidResponse("sync_send".to_string(),Some(err.to_string())))?;
                let mut message = vec![0u8; length];
                self.transports.read_exact_(&mut message).await?;
                let message = std::str::from_utf8(&message)?;
                Err(AdbTransportError::AdbError(message.to_string()).into())
            }
        }
    }
}
