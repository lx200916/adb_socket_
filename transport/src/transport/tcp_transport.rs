use std::{
    path::{Path, PathBuf},
    vec, net::Ipv4Addr,
};

use crate::{AdbCommand, AdbRespStatus, AdbSyncModeCommand, AdbTransportError};
use anyhow::Result;
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use nom::AsBytes;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use super::transport::AdbTransport;

pub struct TcpStreamTransport {
    pub(crate) stream: TcpStream,
    pub(crate) addr: Ipv4Addr,
    pub(crate) port: u16
}
#[async_trait]
impl AdbTransport for TcpStreamTransport {
    async fn read_to_end(&mut self) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        self.stream.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }
    async fn read_buf_(&mut self, buffer: &mut BytesMut) -> Result<usize, AdbTransportError> {
        self.stream
            .read_buf(buffer)
            .await
            .map_err(|err| AdbTransportError::IoError(err))
    }
    async fn read_(&mut self, buffer: &mut [u8]) -> Result<usize, AdbTransportError> {
        self.stream
            .read(buffer)
            .await
            .map_err(|err| AdbTransportError::IoError(err))
    }
    async fn reconnect(&mut self) -> Result<()> {
        self.stream = TcpStream::connect((self.addr,self.port)).await?;
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
        if length.starts_with('\r') {
            return Ok(0);
        }
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
impl TcpStreamTransport {
    pub async fn new(addr: String) -> Result<Self> {
        let (addr,port) = if let Some(index) = addr.find(':') {
            let port = addr[index+1..].parse::<u16>()?;
            let addr = addr[..index].to_string();
            (addr,port)
        }else{
            (addr,5035)
        };
        let addr = addr.parse::<Ipv4Addr>()?;
        
        let stream = TcpStream::connect((addr,port)).await?;
        Ok(Self { stream, addr ,port})
    }
    pub async fn send_request_(&mut self, command: AdbCommand) -> Result<()> {

        let command_string = command.to_string();
        // println!("command_string: {:?}", command_string);
        let command_string = format!("{:04x}{}", command_string.len(), command_string);
        self.stream.write_all(command_string.as_bytes()).await?;
        let mut resp_status = [0u8; 4];
        self.stream.read_exact(&mut resp_status).await?;

        match AdbRespStatus::from(resp_status) {
            AdbRespStatus::Okay => {}
            AdbRespStatus::Fail(_) => {
                match command {
                    AdbCommand::ShellExec(_) => {}
                    AdbCommand::Sync => {}
                    _ => {
                        // println!("{:?}",std::str::from_utf8(self.read_to_end().await?.as_bytes()));
                        let length = self.get_length().await?;
                        // if let Ok(length) = length {
                        let mut message = vec![0u8; length];
                        self.stream.read_exact(&mut message).await?;
                        let message = std::str::from_utf8(&message)?;
                        
                        return Err(AdbTransportError::AdbError(message.to_string()).into());
                        // }
                    }
                }
            }
        }

        Ok(())
    }
}
