use parallel_getter::ParallelGetter;
use reqwest;
use serde::Deserialize;
use std::path::PathBuf;
use std::{
    fs::{create_dir_all, remove_dir_all, remove_file, set_permissions, File, Permissions},
    io::copy,
    path::Path,
};
use zip::ZipArchive;
use std::env;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct UpdateInformation {
    pub hash: String,
    pub changelog: String,
    #[serde(rename = "update-page")]
    pub update_page: String,
    #[serde(rename = "download-page-windows")]
    pub download_page_windows: String,
    #[serde(rename = "download-page-mac")]
    pub download_page_mac: String,
}

pub async fn parallel_download(update_information: UpdateInformation) {
    let mut url = String::new();
    if env::consts::OS == ("windows")
    {
        println!("Windows");
        url = update_information.download_page_windows.as_str().to_string();
    }
    else if env::consts::OS == ("macos")
    {
        println!("Mac");
        url = update_information.download_page_mac.as_str().to_string();
    }

    println!("Downloading files from {}", url);

    create_dir_all("./temp");
    let temp_dir = PathBuf::from("./temp");
    let mut file = File::create("./temp/temp.zip").unwrap();

    ParallelGetter::new(&url, &mut file)
        // Optional path to store the parts.
        .cache_path(temp_dir)
        // Number of theads to use.
        .threads(10)
        // threshold (length in bytes) to determine when multiple threads are required.
        .threshold_parallel(1 * 1024 * 1024)
        // threshold for defining when to store parts in memory or on disk.
        .threshold_memory(10 * 1024 * 1024)
        // Callback for monitoring progress.
        .callback(
            5500,
            Box::new(|progress, total| {
                println!(
                    "{} MiB of {} MiB downloaded",
                    (progress / 1024) / 1024,
                    (total / 1024) / 1024
                );
            }),
        )
        // Commit the parallel GET requests.
        .get()
        .unwrap();
}

pub async fn get_download_information() -> UpdateInformation {
    let update_info = reqwest::get(
        "https://raw.githubusercontent.com/Birdthulu/birdthulu.github.io/master/Update.json",
    )
    .await
    .unwrap()
    .json::<UpdateInformation>()
    .await
    .unwrap();

    update_info
}

pub async fn get_file() -> ZipArchive<File> {
    let path = Path::new("./temp/temp.zip");
    let file = File::open(path).unwrap();
    let zip_file = ZipArchive::new(file).unwrap();
    zip_file
}

pub async fn unzip_file(zip_file: ZipArchive<File>) {
    let mut zip_file = zip_file;

    for i in 0..zip_file.len() {
        let mut file = zip_file.by_index(i).unwrap();
        println!("Extracting: {}", file.name());
        let outpath = file.sanitized_name();

        {
            let comment = file.comment();
            if !comment.is_empty() {
                // println!("File {} comment: {}", i, comment);
            }
        }

        if (&*file.name()).ends_with('/') {
            // println!(
            //     "File {} extracted to \"{}\"",
            //     i,
            //     outpath.as_path().display()
            // );
            create_dir_all(&outpath).unwrap();
        } else {
            // println!(
            //     "File {} extracted to \"{}\" ({} bytes)",
            //     i,
            //     outpath.as_path().display(),
            //     file.size()
            // );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = File::create(&outpath).unwrap();
            copy(&mut file, &mut outfile).unwrap();

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    set_permissions(&outpath, Permissions::from_mode(mode)).unwrap();
                }
            }
        }
    }
    remove_dir_all("./temp");

    if env::consts::OS == ("windows") 
    {
        Command::new("Dolphin.exe")
                .spawn()
                .expect("failed to execute process")
    } 
    else 
    {
        Command::new("Dolphin.app")
                .spawn()
                .expect("failed to execute process")
    };
}
