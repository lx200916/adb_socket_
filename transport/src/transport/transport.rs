use anyhow::Result;
use async_trait::async_trait;

use crate::{AdbCommand, AdbSyncModeCommand};
#[async_trait]
pub trait AdbTransport {
    async fn reconnect(&mut self) -> Result<()>;
    async fn send_command(&mut self, command: AdbCommand, wait_for_resp: bool) -> Result<Vec<u8>>;
    async fn send_sync_command(&mut self, command: AdbSyncModeCommand) -> Result<()>;
}

