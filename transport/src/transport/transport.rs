use anyhow::Result;
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};

use crate::{AdbCommand, AdbSyncModeCommand, AdbTransportError};
#[async_trait]
pub trait AdbTransport {
    async fn reconnect(&mut self) -> Result<()>;
    async fn send_command(&mut self, command: AdbCommand, wait_for_resp: bool) -> Result<Vec<u8>>;
    async fn send_sync_command(
        &mut self,
        command: AdbSyncModeCommand,
    ) -> Result<(), AdbTransportError>;
    async fn get_length(&mut self) -> Result<usize, AdbTransportError>;
    async fn read_exact(&mut self, length: usize) -> Result<Vec<u8>, AdbTransportError>;
    async fn read_exact_(&mut self, buffer: &mut [u8]) -> Result<(), AdbTransportError>;
    async fn read_(&mut self, buffer: &mut [u8]) -> Result<usize, AdbTransportError>;
    async fn read_buf_(&mut self, buffer: &mut BytesMut) -> Result<usize, AdbTransportError>;

    async fn write_all(&mut self, data: &[u8]) -> Result<()>;
}
