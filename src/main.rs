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

use horrorshow::{RenderOnce, RenderBox, RenderMut, Render};
use horrorshow::helper::doctype;
use horrorshow::{html, box_html};

#[derive(StructOpt, Debug)]
#[structopt(author, about)]
struct Configuration {
    #[structopt(short, long, default_value = "Sources/")]
    source_directory: String,

    #[structopt(short, long, default_value = "downloads/")]
    download_directory: String,
}

fn get_document_name<'a>(abb: &'a str) -> &'a str {
    match abb {
        "AR" => "Annual report",
        "FR" => "Financial report",
        "SR" => "Sustainability report",
        "CG" => "Corporate Governance",
        "RS" => "Annual Results",
        "CR" => "Compensation Report",
        "ST" => "Strategy Report",
        "AD" => "Addendum",
        "AM" => "Annual Meeting Minutes",
        "RR" => "Risk Report",
        "RV" => "Review",
        _ => &abb,
    }
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

impl Company {
    fn get_reports(&self, year: u16, language: &str) -> Vec<Report> {
        let mut reps = Vec::new();
        for r in &self.reports {
            if r.year == year && r.language == language {
                reps.push(r.clone());
            }
        }
        reps
    }
}

struct Download {
    size: u64,
    mime_type: String,
}

struct CompanyDownloads {
    company: Company,
    downloads: Vec<Download>,
}

impl CompanyDownloads {
    pub fn get_number_warnings(&self) -> u32 {
        let mut warnings = 0;
        for d in &self.downloads {
            if d.mime_type != "application/pdf" || d.size < 100 {
                warnings += 1;
            }
        }
        warnings
    }
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
    let size = metadata.len() / 1024;
    //let mime_type = tree_magic::from_filepath(&file_path);
    let mime_type = "application/pdf".to_owned();
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
                let company_download = CompanyDownloads{ company, downloads };
                companies.push(company_download);
            }
            None => println!("Error"),
        }
    }
    create_reports(&companies);

    Ok(())
}

fn create_reports(companies: &Vec<CompanyDownloads>) {
    create_index(companies);
    for company in companies {
        create_company_report(company);
    }
}

fn create_index(companies: &Vec<CompanyDownloads>) {
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
                            th {
                                : "Warnings"
                            }                            
                        }
                        @ for company_download in companies {
                            tr {
                                td {
                                    a (href=format_args!("{}.html", company_download.company.name)) {
                                        : &company_download.company.name
                                    }
                                }
                                td {
                                    : &company_download.company.reports.len()
                                }
                                td {
                                    : format_args!("{}-{}", &company_download.company.oldest_year, &company_download.company.newest_year)
                                }
                                td {
                                    : &company_download.get_number_warnings()
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

fn create_company_report(company_download: &CompanyDownloads) {
    let company = &company_download.company;
    let company_name = &company_download.company.name;

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
                                : "EN"
                            }                               
                            th {
                                : "DE"
                            }
                            th {
                                : "FR"
                            }        
                            th {
                                : "IT"
                            }                                                                                                     
                        }
                        @ for year in (company.oldest_year..=company.newest_year).rev() {
                            tr {
                                td {
                                    : year
                                }
                                td {
                                    @ for report in company.get_reports(year, "EN") {
                                        a (href=&report.link, target=&target) {
                                            : get_document_name(&report.report_type)
                                        }
                                        br;
                                    }
                                }
                                td {
                                    @ for report in company.get_reports(year, "DE") {
                                        a (href=&report.link, target=&target) {
                                            : get_document_name(&report.report_type)
                                        }
                                        br;
                                    }
                                }  
                                td {
                                    @ for report in company.get_reports(year, "FR") {
                                        a (href=&report.link, target=&target) {
                                            : get_document_name(&report.report_type)
                                        }
                                        br;
                                    }
                                }  
                                td {
                                    @ for report in company.get_reports(year, "IT") {
                                        a (href=&report.link, target=&target) {
                                            : get_document_name(&report.report_type)
                                        }
                                        br;
                                    }
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
