use parallel_getter::ParallelGetter;
use reqwest;
use serde::Deserialize;
use std::{
    fs::{create_dir_all, set_permissions, File, Permissions},
    io::copy,
    path::Path,
};
use tempfile::tempdir;
use zip::ZipArchive;

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
    let t = update_information.download_page_windows.as_str();
    let url = "http://download1394.mediafire.com/vksc782np9dg/7yt5o4vci83mid0/Project%2B+v2.15+Netplay+%28Windows%29.zip";

    let temp_dir = tempdir().unwrap().into_path();
    let mut file = File::create("pplus.zip").unwrap();

    ParallelGetter::new(url, &mut file)
        // Optional path to store the parts.
        .cache_path(temp_dir)
        // Number of theads to use.
        .threads(10)
        // threshold (length in bytes) to determine when multiple threads are required.
        .threshold_parallel(1 * 1024 * 1024)
        // threshold for defining when to store parts in memory or on disk.
        .threshold_memory(10 * 1024 * 1024)
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
    let path = Path::new("./pplus.zip");
    let file = File::open(path).unwrap();
    let zip_file = ZipArchive::new(file).unwrap();
    zip_file
}

pub async fn unzip_file(zip_file: ZipArchive<File>) {
    let mut zip_file = zip_file;

    for i in 0..zip_file.len() {
        let mut file = zip_file.by_index(i).unwrap();
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
}
