use std::{path::{Path, PathBuf}, vec};

use async_trait::async_trait;
use tokio::{net::UnixStream, io::{AsyncWriteExt, AsyncReadExt}};
use anyhow::Result;
use crate::{AdbCommand, AdbRespStatus, AdbTransportError, AdbSyncModeCommand};

use super::transport::AdbTransport;

pub struct UnixStreamTransport {
    pub(crate) stream: UnixStream,
    pub(crate) addr: PathBuf,
}
#[async_trait]
impl AdbTransport for UnixStreamTransport {
    async fn reconnect(&mut self) -> Result<()> {
        self.stream = UnixStream::connect(&self.addr).await?;
        Ok(())
    }
    async fn send_command(&mut self, command: AdbCommand, wait_for_resp: bool) -> Result<Vec<u8>> {
        self.send_request_(command).await?;
        if wait_for_resp {
            let length = self.get_length().await.map_err(|err|AdbTransportError::InvalidResponse)?;
            let mut message = vec![0u8; length];
            self.stream.read_exact(&mut message).await?;
            return Ok(message);
        }
        Ok(vec![])
    }
    async fn send_sync_command(&mut self, command: AdbSyncModeCommand) -> Result<()>{
        self.stream.write_all(command.to_string().as_bytes()).await?;
        Ok(())
    }
}
impl UnixStreamTransport {
    pub async fn new(addr: String) -> Result<Self> {
        let addr: PathBuf = Path::new(&addr).to_path_buf();
        let stream = UnixStream::connect(addr.clone()).await?;
        Ok(Self {
            stream,
            addr,
        })
    }
    pub async fn send_request_(&mut self,command:AdbCommand)->Result<()>{
        let command_string = command.to_string();
        let command_string = format!("{:04x}{}", command_string.len(), command_string);
        self.stream.write_all(command_string.as_bytes()).await?;
        let mut resp_status = [0u8; 4];
        self.stream.read_exact(&mut resp_status).await?;
        match AdbRespStatus::from(resp_status) {
            AdbRespStatus::Okay => {},
            AdbRespStatus::Fail(_) => {
                let length = self.get_length().await.map_err(|_|AdbTransportError::InvalidResponse)?;
                let mut message = vec![0u8; length];
                self.stream.read_exact(&mut message).await?;
                let message = std::str::from_utf8(&message)?;
                return Err(AdbTransportError::AdbError(message.to_string()).into());
            }
            
        }

        Ok(())
    }
    async fn get_length(&mut self) -> Result<usize> {
        let mut length = [0u8; 4];
        self.stream.read_exact(&mut length).await?;
        let length = std::str::from_utf8(&length)?;
        let length = usize::from_str_radix(length, 16)?;
        Ok(length)
    }

}