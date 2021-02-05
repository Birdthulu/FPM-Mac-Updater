#![allow(non_snake_case)]
mod utils;

use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::consts::OS == ("macos") {
        //working directory is in the home folder, we need to move to where dolphn is
        //env::args()[0] is the directory of the update script, which is 3
        //directories down from where we need to be
        let arg0 = &std::env::args().next().unwrap();
        let path = Path::new(arg0).parent().unwrap().join("../../..");
        println!("new path: {:?}", path);
        std::env::set_current_dir(path)?;

        //we don't want to accidentally overwrite the wrong dolphin app
        //TODO: handle this without panicing
        let app_name = Path::new(arg0).components().rev().nth(3).unwrap().as_os_str();
        if !(app_name == "Dolphin.app") && Path::new("Dolphin.app").exists() {
            panic!("ERROR: you have a different dolphin in this folder named Dolphin.app\n\
                Please rename it or move it to a different folder.");
        }
    }
    println!("pwd: {:?}", std::env::current_dir());
    let download_information = utils::get_download_information().await;
    utils::parallel_download(download_information).await;
    let zip_file = utils::get_file().await;
    utils::unzip_file(zip_file).await;
    Ok(())
}
