use chrono::{Datelike, Utc};
#[macro_use]
extern crate clap;
use clap::{App, Arg};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::Write;
use tokio::prelude::*;

use log::{debug, error, info, trace, warn};
use simplelog::*;

use serde_derive::Deserialize;

use walkdir::WalkDir;

#[macro_use]
extern crate horrorshow;
use horrorshow::helper::doctype;
use horrorshow::prelude::*;

#[derive(Debug, Deserialize)]
enum CompanyType {
    Smi, // Swiss market index
    SmiMid,
    Other,
}

#[derive(Debug, Deserialize)]
struct Company {
    company: String,
    company_type: CompanyType,
}

#[derive(Debug, Deserialize, Clone)]
struct Report {
    company: String,
    language: String,
    report_type: String,
    year: u16,
    link: String,
}

pub fn create_file_list(path: &str, filetype_filter_function: &Fn(&str) -> bool) -> Vec<PathBuf> {
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

async fn download(root_path: &Path, report: Report) -> Result<(), Box<dyn Error>> {
    let file_name = format!("{}-{}.pdf", report.report_type, report.language);

    let path = root_path.join(&report.company);
    let path = path.join(&report.year.to_string());
    fs::create_dir_all(&path)?;
    let file_path = path.join(file_name);
    let file_exists = file_path.exists();
    if !file_exists {
        info!("Processing path: '{:?}'", file_path);

        let mut response = reqwest::get(&report.link).await?;

        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)
            .await?;
        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
        }
    /*if response.status().is_success() {
        copy(&mut text, &mut dest)?;
    } else {
        error!("File {:?} failed.", report.link);
        if let Some(length) = response.content_length() {
            error!("Response length {:?}", length);
        }
        fs::remove_file(fname);
    }*/
    } else {
        debug!("file already exists: '{:?}'", file_path);
        //println!("Try to  open file {:?}", fname);
        /*let mut open_result = Document::load(&fname);
        if let Err(error) = open_result {
            error!("Could not open file {:?}", fname);
        } else if let Ok(document) = open_result {
            let pages = document.get_pages();

            println!("{:?}", pages.len());
        }*/
    }
    Ok(())
}

async fn iterate_files(root_path: PathBuf, file: &File) -> Result<Vec<Report>, Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new().delimiter(b';').from_reader(file);
    let mut future_list = Vec::new();
    let mut reports = Vec::new();

    for result in rdr.deserialize() {
        let report: Report = result?;
        let result = download(&root_path, report.clone());
        future_list.push((report, result));
    }
    for (report, future) in future_list {
        let result = future.await;
        match result {
            Ok(_) => {
                //trace!("{:?}", report);
                reports.push(report);
            }
            Err(e) => error!("Error occurred downloading file {}", e),
        }
    }
    Ok(reports)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //env_logger::init();

    let matches = App::new("Annual report downloader")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Download annual reports from the Internet")
        .arg(
            Arg::with_name("download-directory")
                .short("d")
                .help("Directory into which to download the files")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("source-directory")
                .short("s")
                .help("Directory that contains the data sources")
                .takes_value(true),
        )
        .get_matches();

    let now = Utc::now();
    let date = format!("{}-{:02}-{:02}", now.year(), now.month(), now.day());
    let root_downloads = matches
        .value_of("download-directory")
        .unwrap_or("downloads");
    let download_directory = format!("{}/{}", root_downloads, date);
    let log_file = format!("{}/output.txt", download_directory);
    let root_path = PathBuf::from(&download_directory);
    fs::create_dir_all(&root_path);
    let source_path = Path::new(matches.value_of("source-directory").unwrap_or("Sources"));

    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap(),
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
            let source_file = source_file.unwrap();
            println!("Processing: {}", source_file.path().display());
            let file = File::open(source_file.path()).unwrap();
            let path = PathBuf::from(&my_root_path);
            let result = iterate_files(path, &file).await;
            match result {
                Ok(reports) => Some(reports),
                Err(e) => {
                    error!("Error deserializing file {:?}", file);
                    None
                },
            }
        });
        join_handles.push(join_handle);
    }
    for join_handle in join_handles {
        let result = join_handle.await?;
        match result {
            Some(reports) => print_reports(&reports),
            None => println!("Error"),
        }
    }
    
    /*let is_pdf = |ext: &str | { ext.contains("pdf") };
    println!("{:?}", &download_directory);
    let all_documents = create_file_list(&download_directory, &is_pdf);
    for doc in all_documents {
        let file_name = doc.file_stem().unwrap().to_string_lossy();
        let components: Vec<&str> = file_name.split("-").collect();
        let document_type = components[0].clone();
        let language = components[1].clone();

        let year_folder = doc.parent().unwrap();
        let year = year_folder.file_name().unwrap().to_string_lossy();

        let company_folder = year_folder.parent().unwrap();
        let company = company_folder.file_name().unwrap().to_string_lossy();
        println!("{} {}: {} {}", company, year, document_type, language);
    }
*/
    Ok(())
}

fn print_reports(reports: &Vec<Report>) {
    let company = &reports[0].company;
    let target = "_blank";
    //println!("{} {}", , reports.len());
    let index_content = format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                head {
                    title : company
                }
                body {
                    h1 {
                        : company
                    }
                    table {
                        tr {
                            th {
                                : "Year"
                            }
                            th {
                                : "Type"
                            }
                            th {
                                : "Language"
                            }
                            th {
                                : "Link"
                            }
                        }
                        @ for report in reports {
                            tr {
                                td {
                                    : report.year
                                }
                                td {
                                    : &report.report_type
                                }
                                td {
                                    : &report.language
                                }
                                td {
                                    a (href=&report.link, target=&target) {
                                        : "Link"
                                    }
                                }                                
                            }
                        }
                    }
                }
            }
        }
    );
    let mut index_file = File::create(format!("html/{}.html", &company)).unwrap();
    writeln!(index_file, "{}", index_content).unwrap();
    /*for report in reports {
        println!("{}", report.link);
    }*/
}