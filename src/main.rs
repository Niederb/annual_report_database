use chrono::{Datelike, Utc};
use structopt::StructOpt;

use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::prelude::*;

use log::{debug, error, info};
use simplelog::*;

use serde_derive::Deserialize;

use walkdir::WalkDir;

use horrorshow::helper::doctype;
use horrorshow::html;

#[derive(StructOpt, Debug)]
#[structopt(author, about)]
struct Configuration {
    #[structopt(short, long, default_value = "Sources/")]
    source_directory: String,

    #[structopt(short, long, default_value = "downloads/")]
    download_directory: String,
}

#[derive(Debug, Deserialize)]
enum Language {
    EN,
    DE,
    FR,
    IT,
}

#[derive(Debug, Deserialize)]
struct Company {
    name: String,
    reports: Vec<Report>,
    oldest_year: u16,
    newest_year: u16,
}

struct Download {
    size: u64,
    mime_type: String,
}

impl Company {
    fn new(reports: Vec<Report>) -> Company {
        let name = if reports.len() > 0 {
            reports[0].company.to_owned()
        } else {
            String::new()
        };
        let newest_year = reports.iter().fold(0, |acc, x| std::cmp::max(acc, x.year));
        let oldest_year = reports
            .iter()
            .fold(u16::MAX, |acc, x| std::cmp::min(acc, x.year));
        Company {
            name,
            reports,
            oldest_year,
            newest_year,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Report {
    company: String,
    language: String,
    report_type: String,
    year: u16,
    link: String,
}

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

async fn download(root_path: &Path, report: Report) -> Result<Download, Box<dyn Error>> {
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
            .open(&file_path)
            .await?;
        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
        }
    } else {
        debug!("file already exists: '{:?}'", file_path);
    }
    let metadata = fs::metadata(&file_path)?;
    let size = metadata.len();
    let mime_type = tree_magic::from_filepath(&file_path);
    let d = Download { size, mime_type};
    Ok(d)
}

async fn iterate_files(root_path: PathBuf, file: &File) -> Result<(Company, Vec<Download>), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new().delimiter(b';').from_reader(file);
    let mut future_list = Vec::new();
    let mut reports = Vec::new();
    let mut downloads = Vec::new();

    for result in rdr.deserialize() {
        let report: Report = result?;
        let result = download(&root_path, report.clone());
        future_list.push((report, result));
    }
    for (report, future) in future_list {
        let result = future.await;
        match result {
            Ok(download) => {
                //trace!("{:?}", report);
                reports.push(report);
                downloads.push(download);
            }
            Err(e) => error!("Error occurred downloading file {}", e),
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
            let file = File::open(source_file.path()).expect(&format!("Error opening file {:?}", &source_file.path()));
            let path = PathBuf::from(&my_root_path);
            let result = iterate_files(path, &file).await;
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
            Some((mut company, downloads)) => {
                company.reports.sort_by(|a, b| b.year.cmp(&a.year));
                //create_company_report(&company)
                companies.push((company, downloads));
            }
            None => println!("Error"),
        }
    }
    create_reports(&companies);

    Ok(())
}

fn create_reports(companies: &Vec<(Company, Vec<Download>)>) {
    create_index(companies);
    for company in companies {
        create_company_report(company);
    }
}

fn create_index(companies: &Vec<(Company, Vec<Download>)>) {
    let index_content = format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                head {
                    title : "Annual reports"
                }
                body {
                    h1 {
                        : "Annual reports"
                    }
                    table {
                        tr {
                            th {
                                : "Company"
                            }
                            th {
                                : "Number documents"
                            }
                            th {
                                : "Data range"
                            }
                        }
                        @ for (company, _) in companies {
                            tr {
                                td {
                                    a (href=format_args!("{}.html", company.name)) {
                                        : &company.name
                                    }
                                }
                                td {
                                    : &company.reports.len()
                                }
                                td {
                                    : format_args!("{}-{}", &company.oldest_year, &company.newest_year)
                                }
                            }
                        }
                    }
                }
            }
        }
    );
    let mut index_file = File::create("html/index.html").unwrap();
    writeln!(index_file, "{}", index_content).unwrap();
}

fn create_company_report(company: &(Company, Vec<Download>)) {
    let reports = &company.0.reports;
    let downloads = &company.1;

    let company_name = &company.0.name;

    let documents: Vec<(&Report, &Download)> = reports.into_iter()
        .zip(downloads.into_iter())
        .collect();

    let target = "_blank";

    let index_content = format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                head {
                    title : company_name
                }
                body {
                    h1 {
                        : company_name
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
                            th {
                                : "Size"
                            }
                            th {
                                : "Type"
                            }                            
                        }
                        @ for document in &documents {
                            tr {
                                td {
                                    : document.0.year
                                }
                                td {
                                    : &document.0.report_type
                                }
                                td {
                                    : &document.0.language
                                }
                                td {
                                    a (href=&document.0.link, target=&target) {
                                        : "Link"
                                    }
                                }
                                td {
                                    : &document.1.size
                                }              
                                td {
                                    : &document.1.mime_type
                                }                                                   
                            }
                        }
                    }
                }
            }
        }
    );
    let mut index_file = File::create(format!("html/{}.html", &company_name)).unwrap();
    writeln!(index_file, "{}", index_content).unwrap();
}
