use crate::utils::check_path;
use crate::{AdbCommand, AdbTransportError, AdbTransports};
use anyhow::{Ok, Result};
use bytes::{buf, Bytes};
impl AdbTransports {
    #[async_backtrace::framed]
    pub async fn shell<S: ToString>(
        &mut self,
        serial: Option<S>,
        cmd: Vec<String>,
        callback: impl Fn(Vec<u8>),
    ) -> Result<()> {
        self.may_set_serial(serial).await?;

        let cmd = cmd
        //     .into_iter()
        //     .map(|str| str.replace(" ", "\\ "))
        //     .collect::<Vec<String>>()
            .join(" ");
        // println!("cmd {:?}", cmd.clone());

        self.transports
            .send_command(AdbCommand::ShellExec(cmd), false)
            .await?;
        let mut buffer = [0u8;100];
        loop{
            let read = self.transports.read_(&mut buffer).await?;
            if read == 0 {
                break;
            }
            callback(buffer[..read].to_vec());
        }
        // let mut line = Vec::new();
        // let mut buffer = bytes::BytesMut::new();
        // loop {
        //     let read = self.transports.read_buf_(&mut buffer).await?;
        //     // println!("read {}",read);
        //     if read == 0 {
        //         if !line.is_empty() {
        //             let lines: &str = std::str::from_utf8(&line)
        //                 .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        //             callback(lines.to_string());
        //         }
        //         break;
        //     }
        //     while let Some(i) = buffer.iter().position(|&x| x == b'\n') {
        //         line.extend_from_slice(&buffer.split_to(i + 1));
        //         let lines: &str = std::str::from_utf8(&line)
        //             .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        //         callback(lines.to_string());
        //         line.clear();
        //     }
        // }

        Ok(())
    }
    #[async_backtrace::framed]
    pub async fn mkdir<S: ToString, A: AsRef<str>>(
        &mut self,
        serial: Option<S>,
        path: A,
    ) -> Result<Vec<u8>> {
        let path = path.as_ref();
        let res_ = check_path(path.to_string())?;
        if !res_{
            return Err(anyhow::anyhow!("Path is not allowed"));
        }

        self.may_set_serial(serial).await?;

        let cmd = format!("mkdir {}", path);
        self.transports
            .send_command(AdbCommand::ShellExec(cmd), false)
            .await
        
    }
}
