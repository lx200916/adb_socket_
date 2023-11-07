mod devices;
mod list;
mod pull;
mod push;
mod shell;
mod stat;

#[cfg(test)]
mod command_test {
    use crate::{result::stat::FileType, AdbTransports};
    use anyhow::Result;
    async fn new_transport() -> Result<AdbTransports> {
        Ok(AdbTransports::new("/var/run/adb.sock".to_string(), false).await?)
    }
    #[tokio::test]
    #[async_backtrace::framed]
    async fn test_list() {
        let mut transport = new_transport().await.unwrap();
        let serial: Option<String> = None;
        let list = transport.list("/sdcard".to_string(), serial).await.unwrap();
        println!("{:?}", list);
    }
    #[tokio::test]
    async fn test_stat_non_exist() {
        let mut transport = new_transport().await.unwrap();
        let serial: Option<String> = None;
        let stat = transport
            .sync_stat("/sdcard/NON_EXITST_XXXXX".to_string(), serial)
            .await
            .unwrap();
        println!("{:?}", stat.get_file_type());
        assert_eq!(stat.get_file_type(), FileType::Other)
    }
    #[tokio::test]
    async fn test_stat_dic() {
        let mut transport = new_transport().await.unwrap();
        let serial: Option<String> = None;
        let stat = transport
            .sync_stat("/sdcard/Download".to_string(), serial)
            .await
            .unwrap();
        println!("{:?}", stat.get_file_type());
        assert_eq!(stat.get_file_type(), FileType::Directory)
    }
    #[tokio::test]
    async fn test_stat_file() {
        let mut transport = new_transport().await.unwrap();
        let serial: Option<String> = None;
        let stat = transport
            .sync_stat("/system/build.prop".to_string(), serial)
            .await
            .unwrap();
        println!("{:?}", stat.get_file_type());
        assert_eq!(stat.get_file_type(), FileType::File)
    }
}
