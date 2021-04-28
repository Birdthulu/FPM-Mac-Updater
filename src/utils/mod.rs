use reqwest;
use serde::Deserialize;
use std::{
    fs::{create_dir_all, remove_dir_all, File},
    io::copy,
    path::Path,
};
use std::ffi::OsStr;
use zip::ZipArchive;
use std::env;
use std::process::Command;
use std::io::prelude::*;

#[derive(Debug, Deserialize)]
pub struct UpdateInformation {
    pub hash: String,
    pub changelog: String,
    #[serde(rename = "updater-update")]
    pub updater_update: Option<String>,
    #[serde(rename = "update-page")]
    pub update_page: String,
    #[serde(rename = "download-page-windows")]
    pub download_page_windows: String,
    #[serde(rename = "download-page-mac")]
    pub download_page_mac: String,
}

pub async fn download(update_information: UpdateInformation) {
    let mut url = String::new();
    if env::consts::OS == ("windows")
    {
        url = update_information.download_page_windows.as_str().to_string();
    }
    else if env::consts::OS == ("macos")
    {
        url = update_information.download_page_mac.as_str().to_string();
    }

    println!("Downloading files from {}", url);

    //TODO use system temp dir
    create_dir_all("./temp").expect("Could not create file");

    let mut file = File::create("./temp/temp.zip").unwrap();
    let mut res = reqwest::get(&url).await.expect("failed to download update file");

    let mut last_update_time = std::time::Instant::now();
    let file_size = res.content_length();
    let mut downloaded = 0;

    while let Some(chunk) = res.chunk().await.expect("error downloading update file") {
        file.write_all(&chunk).expect("error saving update file");

        //print download progress
        downloaded += chunk.len();
        let now = std::time::Instant::now();
        if now.duration_since(last_update_time) > std::time::Duration::new(1, 0) {
            last_update_time = now;
            let file_size = file_size.map(|n| ((n / 1000) as f64 / 1000.).to_string()).unwrap_or(String::from("Unknown"));
            println!("{} MB out of {} MB downloaded", (downloaded / 1000) as f64 / 1000., file_size);
        }
    }

}

pub async fn get_download_information(update_json_url: &str) -> UpdateInformation {
    println!("getting update info from {}", update_json_url);
    let update_info = reqwest::get(
        update_json_url,
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

//merge one directory into another, overwriting files if they exist in both source and dest
//but leaving files that only exist in dest
//should be a library function imo
fn merge_dir_recursively(source: &Path, dest: &Path) -> Result<(), std::io::Error> {
    let new_dest = dest.join(source.file_name().unwrap());
    //println!("creating {:?}", &new_dest);
    create_dir_all(&new_dest)?;
    
    for source_file in std::fs::read_dir(source)? {
        let new_path = source_file?.path();
        //println!("{:?}", new_path);
        if new_path.is_dir() {
            merge_dir_recursively(&new_path, &new_dest)?;
        }
        else {
            //println!("renaming {:?} to {:?}", &new_path, &new_dest.join(&new_path.file_name().unwrap()));
            std::fs::rename(&new_path, &new_dest.join(&new_path.file_name().unwrap()))?;
        }
    };
    //println!("removing {:?}", source);
    std::fs::remove_dir(source)?;
    Ok(())
}

pub async fn unzip_file(zip_file: ZipArchive<File>, dolphin_name: &std::ffi::OsStr) {
    let mut zip_file = zip_file;

    let extract_dir = Path::new("./temp/ext");
    std::fs::create_dir_all(extract_dir).unwrap();
    
    for i in 0..zip_file.len() {
        let mut file = zip_file.by_index(i).unwrap();
        println!("Extracting: {}", file.name());
        let outpath = extract_dir.join(file.sanitized_name());

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
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }
    }

    //rename Dolphin.app to the name of the dolphin that launched the updater
    //so that we don't overwrite the wrong dolphin.
    if extract_dir.join("Dolphin.app").exists() && dolphin_name != "Dolphin.app" {
        let new_path = extract_dir.join(dolphin_name);
        assert!(!new_path.exists());
        println!("renaming {:?} to {:?}", extract_dir.join("Dolphin.app"), &new_path);
        std::fs::rename(extract_dir.join("Dolphin.app"), &new_path).unwrap();
    }

    //merge everything from the extraction folder into the folder where dolphin is.
    for file in std::fs::read_dir(extract_dir).unwrap() {
        let path = file.unwrap().path();
        if path.is_dir() {
            println!("merging {:?}", &path.file_name().unwrap_or(OsStr::new("")));
            merge_dir_recursively(&path, Path::new(".")).unwrap();
        }
        else {
            println!("copying {:?}", path.file_name().unwrap_or(OsStr::new("")));
            std::fs::rename(&path, path.file_name().unwrap()).unwrap();
        }
    }

    //TODO run this after errors, don't just crash
    remove_dir_all("./temp").expect("Could not delete file");

    //launch dolphin
    println!("launching {:?}", Path::new(dolphin_name).join("Contents/MacOS/Dolphin"));
    let path = std::fs::canonicalize(Path::new(dolphin_name).join("Contents/MacOS/Dolphin"))
            .expect("failed to find executable");
    Command::new(path)
            .spawn()
            .expect("failed to execute process");
}
