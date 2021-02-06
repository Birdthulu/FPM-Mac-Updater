#![allow(non_snake_case)]
mod utils;

use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    //working directory is in the home folder, we need to move to where dolphn is
    //env::args()[0] is the directory of the update script, which is 3
    //directories down from where we need to be
    let arg0 = &std::env::args().next().unwrap();
    let path = Path::new(arg0).parent().unwrap().join("../../..");
    println!("new path: {:?}", path);
    std::env::set_current_dir(path)?;

    let app_name = Path::new(arg0).components().rev().nth(3).unwrap().as_os_str();

    println!("pwd: {:?}", std::env::current_dir());
    let download_information = utils::get_download_information().await;
    utils::parallel_download(download_information).await;
    let zip_file = utils::get_file().await;
    utils::unzip_file(zip_file, app_name).await;
    Ok(())
}
