use anyhow::Result;
use clap::Parser;
use serde::Serialize;
use std::path::{Path, PathBuf};
use transport::{result::stat::FileType, AdbTransports};
use transport::result::device::Devices;
#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Arguments {
    /// Serial number of the device to connect to
    #[clap(short, long)]
    pub serial: Option<String>,
    /// Unix socket to connect to server.
    #[clap(long, default_value = "/var/run/adb.sock")]
    pub socket: String,
    /// Use JSON output
    #[clap(long)]
    pub json: bool,
    #[clap(subcommand)]
    pub command: SubCommand,
}
#[derive(Parser, Debug)]
pub enum SubCommand {
    /// List connected devices
    Devices {
        /// List devices in long format
        #[clap(short, long)]
        long: bool,
    },
    /// Run a shell command on a device
    Shell {
        /// Command to run
        command: Vec<String>,
    },
    /// Push a file to a device
    Push {
        /// Local path to file
        path: String,
        /// Remote path to file
        filename: String,
    },
    /// Pull a file from a device
    Pull {
        /// Remote path to file
        path: String,
        /// Local path to file
        filename: String,
    },
}

#[derive(Debug, Clone)]
struct FileItem {
    remote_file: PathBuf,
    local_path: PathBuf,
}
#[tokio::main]
async fn main() {
    let args = Arguments::parse();
    let mut adb = transport::AdbTransports::new(args.socket, args.json)
        .await
        .unwrap();
    match args.command {
        SubCommand::Devices { long } => {
            let devices = if long {
                adb.devices_long().await.unwrap()
            } else {
                adb.devices().await.unwrap()
            };
            

            match devices {
                Devices::Devices(devices) => {
                    // devices.iter().for_each(|device| {
                    //     println!("{:?}", device);
                    // });
                    print!("{}", serde_json::to_string(&devices).unwrap());
                }
                Devices::Raw(raw) => {
                    println!("List of devices attached");
                    println!("{}", raw);
                }
            }
        }
        SubCommand::Shell { command } => {
            let callback = |str: Vec<u8>| {
                std::io::Write::write_all(&mut std::io::stdout(), &str).unwrap();
            };
            adb.shell(args.serial, command, callback).await.unwrap();
        }
        SubCommand::Push { path, filename } => {
            let path = Path::new(&path);
            println!("{:?}", path);

            // Check if path exists and is a folder?
            if !path.exists() {
                println!("Path does not exist");
                return;
            }
            if path.is_file() {
                let file = std::fs::File::open(path).unwrap();
                let mut reader = std::io::BufReader::new(file);
                adb.push(args.serial, &mut reader, filename).await.unwrap();
            } else if path.is_dir() {
                //check if `filename` is a dir?
                let remote_type = adb
                    .sync_stat(filename.clone(), args.serial.clone())
                    .await
                    .unwrap()
                    .get_file_type();
                if remote_type != FileType::Directory && remote_type != FileType::Other {
                    println!("can not push folder to a file:{}", filename);
                    return;
                }
                let filename = Path::new(&filename);
                // Walk the directory recursively and push all files, mkdir all folders
                let walker = walkdir::WalkDir::new(path);
                let base = path;
                for entry in walker.into_iter().filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() && !path.is_symlink() {
                        let file = std::fs::File::open(path).unwrap();
                        let mut reader = std::io::BufReader::new(file);
                        // get the remote path of the file, relative to the base path,then concat with  remote filename
                        let filename_ = path.strip_prefix(base).unwrap();
                        let filename_ = filename.join(filename_);
                        adb.push(
                            args.serial.clone(),
                            &mut reader,
                            filename_.to_str().unwrap(),
                        )
                        .await
                        .unwrap();
                        println!(
                            "push file:{} to  {}",
                            filename.to_str().unwrap(),
                            filename_.to_str().unwrap()
                        )
                    } else if path.is_dir() {
                        let filename_ = path.strip_prefix(base).unwrap();
                        let filename_ = filename.join(filename_);
                        // println!("mkdir {}", filename_.to_str().unwrap());

                        // adb.mkdir(args.serial.clone(), filename.to_str().unwrap())
                        //     .await
                        //     .unwrap();
                    }
                }
            }
        }
        SubCommand::Pull { path, filename } => {
            let stat_info = adb
                .sync_stat(path.clone(), args.serial.clone())
                .await
                .unwrap();
            let remote_type = stat_info.get_file_type();

            if remote_type == FileType::Other {
                println!("Remote file does not exist");
                return;
            }
            let filename_: PathBuf = PathBuf::from(filename.clone());
            if remote_type == FileType::Directory {
                if filename_.exists() && !filename_.is_dir() {
                    println!("Pulling A Directory to A File is not supported yet");
                    return;
                }
                println!("Pulling A Directory is not supported yet");
                let mut dirs = vec![PathBuf::from(filename.clone())];
                let mut files = Vec::new();
                walk_remote_dirs(
                    &mut dirs,
                    &mut files,
                    &PathBuf::from(path.clone()),
                    &PathBuf::from(filename.clone()),
                    &mut adb,
                    args.serial.clone(),
                )
                .await
                .unwrap();
            // dbg!(dirs);
            // dbg!(files);
                walk_pull(dirs, files, args.serial.clone(), &mut adb).await.unwrap();

                return;
            }
            let remote_file = PathBuf::from(path.clone());
            let remote_file = remote_file.file_name().unwrap();

            let filename_ = if filename_.exists() {
                if filename_.is_file() {
                    println!("File already exists");
                    filename_.to_path_buf()
                } else if filename_.is_dir() {
                    let new_filename = filename_.join(remote_file);
                    new_filename
                } else {
                    Err(anyhow::anyhow!("Path is not allowed")).unwrap()
                }
            } else {
                filename_
            };
            let mut file = std::fs::File::create(filename_).unwrap();
            adb.pull(args.serial, path.clone(), &mut file)
                .await
                .unwrap();

            {
                println!("Pulled file {} to {}", path, filename);
            }
        }
    }
}
#[inline]
async fn walk_pull(
    dir: Vec<PathBuf>,
    files: Vec<FileItem>,
    serial: Option<String>,
    adb: &mut AdbTransports,
) -> Result<()> {
    for folder in dir {
        std::fs::create_dir_all(folder).expect("create folder failed");
    }
    for f in files {
        let mut file = std::fs::File::create(f.local_path).unwrap();
        adb.pull(serial.clone(), f.remote_file.to_str().unwrap(), &mut file)
            .await
            .unwrap();
                    {
                println!("Pulled file {} to {}", f.remote_file.to_str().unwrap(), f.remote_file.to_str().unwrap());
            }
    }
    Ok(())
}
#[inline]
#[async_recursion::async_recursion(?Send)]
async fn walk_remote_dirs(
    dir: &mut Vec<PathBuf>,
    files: &mut Vec<FileItem>,
    rpath: &PathBuf,
    lpath: &PathBuf,
    adb: &mut AdbTransports,
    serial: Option<String>,
) -> Result<()> {
    let mut walker = adb
        .list(rpath.to_str().unwrap().to_string(), serial.clone())
        .await?;
    for item in walker {
        let file_type = FileType::from(item.mode);
        if file_type == FileType::Directory {
            // println!("dir:{:?}", item.name);
            if item.name == "." || item.name == ".." {
                continue;
            }
            dir.push(lpath.join(item.name.clone()));
            walk_remote_dirs(
                dir,
                files,
                &rpath.join(item.name.clone()),
                &lpath.join(item.name.clone()),
                adb,
                serial.clone(),
            )
            .await?;
        } else if file_type == FileType::File {
            files.push(FileItem {
                remote_file: rpath.join(item.name.clone()),
                local_path: lpath.join(item.name.clone()),
            });
        }
        //TODO: We Do not Care Others.
    }
    Ok(())
}
