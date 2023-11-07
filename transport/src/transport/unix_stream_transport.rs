use std::{
    path::{Path, PathBuf},
    vec,
};

use crate::{AdbCommand, AdbRespStatus, AdbSyncModeCommand, AdbTransportError};
use anyhow::Result;
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use super::transport::AdbTransport;

pub struct UnixStreamTransport {
    pub(crate) stream: UnixStream,
    pub(crate) addr: PathBuf,
}
#[async_trait]
impl AdbTransport for UnixStreamTransport {
    async fn read_buf_(&mut self, buffer: &mut BytesMut) -> Result<usize, AdbTransportError> {
        self.stream.read_buf(buffer).await.map_err(|err|AdbTransportError::IoError(err))
    }
    async fn read_(&mut self, buffer: &mut [u8]) -> Result<usize, AdbTransportError> {
        self.stream
            .read(buffer)
            .await
            .map_err(|err| AdbTransportError::IoError(err))
    }
    async fn reconnect(&mut self) -> Result<()> {
        self.stream = UnixStream::connect(&self.addr).await?;
        Ok(())
    }
    #[async_backtrace::framed]
    async fn send_command(&mut self, command: AdbCommand, wait_for_resp: bool) -> Result<Vec<u8>> {
        self.send_request_(command).await?;
        if wait_for_resp {
            let length = self.get_length().await?;
            let mut message = vec![0u8; length];
            self.stream.read_exact(&mut message).await?;
            return Ok(message);
        }
        Ok(vec![])
    }

    #[async_backtrace::framed]
    async fn send_sync_command(
        &mut self,
        command: AdbSyncModeCommand,
    ) -> Result<(), AdbTransportError> {
        self.stream
            .write_all(command.to_string().as_bytes())
            .await
            .map_err(|err| AdbTransportError::IoError(err))?;
        Ok(())
    }
    #[async_backtrace::framed]
    async fn get_length(&mut self) -> Result<usize, AdbTransportError> {
        let mut length = [0u8; 4];
        self.stream
            .read_exact(&mut length)
            .await
            .map_err(|err| AdbTransportError::IoError(err))?;
        let length = std::str::from_utf8(&length).map_err(|err| {
            AdbTransportError::InvalidResponse(String::from("get_length"), Some(err.to_string()))
        })?;
        let length = usize::from_str_radix(length, 16).map_err(|err| {
            AdbTransportError::InvalidResponse(
                String::from(format!("get_length {:?}", length)),
                Some(err.to_string()),
            )
        })?;
        Ok(length)
    }
    #[async_backtrace::framed]
    async fn read_exact(&mut self, length: usize) -> Result<Vec<u8>, AdbTransportError> {
        let mut message = vec![0u8; length];
        self.stream
            .read_exact(&mut message)
            .await
            .map_err(|err| AdbTransportError::IoError(err))?;
        Ok(message)
    }
    #[async_backtrace::framed]
    async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.stream.write_all(data).await?;
        Ok(())
    }
    #[async_backtrace::framed]
    async fn read_exact_(&mut self, buffer: &mut [u8]) -> Result<(), AdbTransportError> {
        self.stream
            .read_exact(buffer)
            .await
            .map_err(|err| AdbTransportError::IoError(err))?;
        Ok(())
    }
}
impl UnixStreamTransport {
    pub async fn new(addr: String) -> Result<Self> {
        let addr: PathBuf = Path::new(&addr).to_path_buf();
        let stream = UnixStream::connect(addr.clone()).await?;
        Ok(Self { stream, addr })
    }
    pub async fn send_request_(&mut self, command: AdbCommand) -> Result<()> {
        let command_string = command.to_string();
        let command_string = format!("{:04x}{}", command_string.len(), command_string);
        self.stream.write_all(command_string.as_bytes()).await?;
        let mut resp_status = [0u8; 4];
        self.stream.read_exact(&mut resp_status).await?;
        match AdbRespStatus::from(resp_status) {
            AdbRespStatus::Okay => {}
            AdbRespStatus::Fail(_) => {
                let length = self.get_length().await?;
                let mut message = vec![0u8; length];
                self.stream.read_exact(&mut message).await?;
                let message = std::str::from_utf8(&message)?;
                return Err(AdbTransportError::AdbError(message.to_string()).into());
            }
        }

        Ok(())
    }
}
