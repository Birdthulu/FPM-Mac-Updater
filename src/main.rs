mod utils;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let download_information = utils::get_download_information().await;
    utils::parallel_download(download_information).await;
    let zip_file = utils::get_file().await;
    utils::unzip_file(zip_file).await;
    Ok(())
}
