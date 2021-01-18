#![allow(non_snake_case)]
mod utils;

use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::consts::OS == ("macos") {
        let arg0 = &std::env::args().next().unwrap();
        let path = Path::new(arg0).parent().unwrap().join("../..");
        println!("new path: {:?}", path);
        std::env::set_current_dir(path);
    }
    println!("pwd: {:?}", std::env::current_dir());
    let download_information = utils::get_download_information().await;
    utils::parallel_download(download_information).await;
    let zip_file = utils::get_file().await;
    utils::unzip_file(zip_file).await;
    Ok(())
}
