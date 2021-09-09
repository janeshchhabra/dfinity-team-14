extern crate env_logger;
extern crate getopts;
extern crate log;
extern crate tftp_server;

use getopts::Options;
use reqwest::Url;
use std::env;
use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use tftp_server::server::TftpServerBuilder;

fn download_urls(url_list: String, serve_dir: &PathBuf) -> Result<(), String> {
    let urls_s: Vec<&str> = url_list.split(",").collect();

    for url_s in &urls_s {
        // Parse the URL
        let url_r = Url::from_str(url_s)
            .map_err(|e| format!("Failed to parse url: {:?}/{:?}", url_s, e))?;
        let segments = url_r.path_segments()
            .ok_or_else(|| "URL missing file path")?;
        let segments: Vec<&str> = segments.collect();
        if segments.len() != 1 {
            return Err(format!("Invalid file name in URL: {:?}", url_s));
        }
        let file_name = segments[0];
        if file_name.is_empty() {
            return Err(format!("Missing file name in URL: {:?}", url_s));
        }
        println!("URL = {:?}, file_name = {}", url_s, file_name);

        // Download the image
        let start = std::time::Instant::now();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let url_rc = url_r.clone();
        let bytes = rt.block_on(async {
            let ret = reqwest::Client::new()
                .get(url_rc)
                .send()
                .await;
            if let Err(e) = ret {
                return Err(format!("Failed to download image: {:?}/{:?}", url_s, e));
            }

            ret.unwrap().bytes().await.map_err(
                |e| format!("Failed to download image: {:?}/{:?}", url_s, e)
            )
        })?;
        println!("Successfully downloaded {:?}: {} bytes in {:?}",
                 url_s, bytes.len(), start.elapsed());

        // Save the image
        write_image(serve_dir, file_name, &bytes[..])?;
    }
    Ok(())
}

fn write_image(serve_dir: &PathBuf, file_name: &str, bytes: &[u8]) -> Result<(), String> {
    let mut file_path = serve_dir.clone();
    file_path.push(file_name);

    let mut f = File::create(file_path.clone())
        .map_err(|e| format!("Failed to create file {:?}/{:?}", file_path, e))?;
    f.write(bytes)
        .map_err(|e| format!("Failed to write file {:?}/{:?}", file_path, e))?;
    println!("Saved image: {:?}", file_path);
    Ok(())
}

fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("p", "port", "Sets the port the server runs on", "PORT");
    opts.optopt(
        "d",
        "directory",
        "Sets the directory the server serves files on",
        "PATH",
    );
    opts.optopt(
        "u",
        "urls",
        "The comma separated list of image URLs to download",
        "URLS",
    );
    opts.optflag("h", "help", "Print help menu");
    let matches = opts.parse(&args[1..]).unwrap();
    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options]", &program);
        print!("{}", opts.usage(&brief));
        return;
    }

    let socket_addr = matches
        .opt_str("p")
        .map(|p| format!("127.0.0.1:{}", p))
        .map(|addr| SocketAddr::from_str(addr.as_str()).expect("Error parsing address"));
    let dir = matches.opt_str("d").map(|s| PathBuf::from(s));

    // Parse/download the URLs into the specified folder
    let image_urls = matches
        .opt_str("u")
        .unwrap();

    if let Some(path) = &dir {
        download_urls(image_urls, path).expect("Failed to download the images");
    }

    let mut server = TftpServerBuilder::new()
        .addr_opt(socket_addr)
        .serve_dir_opt(dir)
        .build()
        .expect("Error creating server");
    println!(
        "Server created at address {:?}",
        server.local_addr().unwrap()
    );

    match server.run() {
        Ok(_) => println!("Server completed successfully!"),
        Err(e) => println!("Error: {:?}", e),
    }
}
