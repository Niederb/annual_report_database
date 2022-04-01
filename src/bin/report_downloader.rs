use chrono::{Datelike, Utc};
use std::error::Error;
use std::fs;
use std::fs::File;
use structopt::StructOpt;

use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

use log::{debug, error, info};
use simplelog::*;

use reqwest::Client;
use walkdir::WalkDir;

use annual_report_database::data_structures::*;
use annual_report_database::*;

pub fn create_file_list(
    path: &str,
    filetype_filter_function: &dyn Fn(&str) -> bool,
) -> Vec<PathBuf> {
    let mut file_list = Vec::new();
    let walker = WalkDir::new(path).into_iter();

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.into_path();
        if let Some(os_ext) = path.extension() {
            if let Some(ext) = os_ext.to_str() {
                if filetype_filter_function(ext) {
                    file_list.push(path);
                }
            }
        }
    }
    file_list
}

async fn reqwest_download(
    link: &str,
    file_path: &Path,
    client: &Client,
) -> Result<(), Box<dyn Error>> {
    let mut response = client.get(link).send().await?;

    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&file_path)
        .await?;

    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk).await?;
    }
    Ok(())
}

async fn download(
    root_path: &Path,
    report: Report,
    client: &Client,
) -> Result<Download, Box<dyn Error>> {
    let file_path = report.get_file_path(root_path);
    fs::create_dir_all(&file_path.parent().unwrap())?;
    let file_exists = file_path.exists();
    //println!("{:?}", file_path);
    if !file_exists {
        info!("Processing path: '{:?}'", file_path);
        //println!("{}", report.link);
        let response = reqwest_download(&report.link, &file_path, client).await;
        match response {
            Ok(_) => {}
            Err(_) => {
                error!("Deleting file {:?}", file_path);
                std::fs::remove_file(&file_path)?
            }
        }
    } else {
        debug!("file already exists: '{:?}'", file_path);
    }
    let metadata = fs::metadata(&file_path)?;
    let size = metadata.len() / 1024;
    //let mime_type = tree_magic::from_filepath(&file_path);
    let mime_type = "application/pdf".to_owned();
    let d = Download {
        report,
        size,
        mime_type,
    };
    Ok(d)
}

async fn iterate_files(
    root_path: PathBuf,
    file: &File,
    client: &Client,
) -> Result<(Company, Vec<Download>), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new().delimiter(b';').from_reader(file);
    let mut future_list = Vec::new();
    let mut reports = Vec::new();
    let mut downloads = Vec::new();

    for result in rdr.deserialize() {
        let report: Report = result?;
        let result = download(&root_path, report.clone(), client);
        future_list.push((report, result));
    }
    for (report, future) in future_list {
        let result = future.await;
        match result {
            Ok(download) => {
                reports.push(report);
                downloads.push(download);
            }
            Err(e) => error!(
                "Error occurred downloading file {:?}: {}",
                report.get_file_path(&root_path),
                e
            ),
        }
    }
    let company = Company::new(reports);
    Ok((company, downloads))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let c = Configuration::from_args();

    let now = Utc::now();
    let date = format!("{}-{:02}-{:02}", now.year(), now.month(), now.day());

    let download_directory = format!("{}/{}", c.download_directory, date);
    let log_file = format!("{}/output.txt", download_directory);
    let root_path = PathBuf::from(&download_directory);
    fs::create_dir_all(&root_path).unwrap();
    let source_path = Path::new(&c.source_directory);

    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed),
        WriteLogger::new(
            LevelFilter::Error,
            Config::default(),
            File::create(log_file).unwrap(),
        ),
    ])
    .unwrap();

    println!(
        "Downloading into {:?} from source directory {:?}",
        root_path, source_path
    );
    let paths = fs::read_dir(source_path).unwrap();

    let mut join_handles = Vec::new();
    for source_file in paths {
        let my_root_path = root_path.clone();
        let join_handle = tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .build()
                .expect("Failed creating a client");
            let source_file = source_file.unwrap();
            println!("Processing: {}", source_file.path().display());
            let file = File::open(source_file.path())
                .unwrap_or_else(|_| panic!("Error opening file {:?}", &source_file.path()));
            let path = PathBuf::from(&my_root_path);
            let result = iterate_files(path, &file, &client).await;
            match result {
                Ok(reports) => Some(reports),
                Err(_e) => {
                    error!("Error deserializing file {:?}", file);
                    None
                }
            }
        });
        join_handles.push(join_handle);
    }
    let mut companies = Vec::new();
    for join_handle in join_handles {
        let result = join_handle.await?;
        match result {
            Some((company, mut downloads)) => {
                downloads.sort_by(|a, b| b.report.year.cmp(&a.report.year));
                let company_download = CompanyDownloads { company, downloads };
                companies.push(company_download);
            }
            None => println!("Error"),
        }
    }

    let tags = ["SMI", "SMIM", "Bank", "Kantonalbank", "Insurance"];
    let empty_tags = Vec::<&str>::new();
    for t in &tags {
        let smi_list = filter_companies(t, &companies);
        let path = format!("html/{}.html", &t);
        reporting::create_index(&path, &smi_list, &empty_tags);
    }
    reporting::create_reports(&companies, &tags);
    extraction::extract_text(&root_path, &companies);

    Ok(())
}
