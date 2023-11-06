use anyhow::Result;
use async_trait::async_trait;

use crate::{AdbCommand, AdbSyncModeCommand};
#[async_trait]
pub trait AdbTransport {
    async fn reconnect(&mut self) -> Result<()>;
    async fn send_command(&mut self, command: AdbCommand, wait_for_resp: bool) -> Result<Vec<u8>>;
    async fn send_sync_command(&mut self, command: AdbSyncModeCommand) -> Result<()>;
    async fn get_length(&mut self) -> Result<usize>;
    async fn read_exact(&mut self, length: usize) -> Result<Vec<u8>>;
    async fn read_exact_(&mut self, buffer: &mut [u8]) -> Result<(),std::io::Error>;
   
    async fn write_all(&mut self, data: &[u8]) -> Result<()>;
}

