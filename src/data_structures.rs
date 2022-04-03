use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

use crate::reporting::write_metadata;

#[derive(StructOpt, Debug)]
#[structopt(author, about)]
pub struct Configuration {
    #[structopt(short, long, default_value = "Sources/")]
    pub source_directory: String,

    #[structopt(short, long, default_value = "downloads/")]
    pub download_directory: String,
}

pub fn get_document_name(abb: &str) -> &str {
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
        "PS" => "Proxy Statement",
        "10K" => "SEC Form 10-K",
        "20F" => "SEC Form 20-F",
        "GRI" => "GRI Sustainability Reporting Standard",
        _ => abb,
    }
}

pub fn get_language(language: &str) -> &str {
    match language {
        "EN" => "English",
        "DE" => "Deutsch",
        "FR" => "Français",
        "IT" => "Italiano",
        _ => "",
    }
}

#[derive(Debug, Deserialize)]
pub enum Language {
    EN,
    DE,
    FR,
    IT,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Report {
    pub company: String,
    pub language: String,
    pub report_type: String,
    pub year: u16,
    pub link: String,
}

impl Report {
    pub fn get_file_path(&self, root_path: &Path) -> PathBuf {
        let file_name = format!("{}-{}.pdf", self.report_type, self.language);

        let path = root_path.join(&self.company);
        let path = path.join(&self.year.to_string());
        //fs::create_dir_all(&path).unwrap();
        path.join(file_name)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompanyMetadata {
    pub name: String,
    pub country: String,
    pub tags: Vec<String>,
    pub comment: String,
    pub links: Vec<String>,
    pub annual_closing_date: String,
    pub accounting_rules: String,
    pub legal_form: String,
    pub url: String,
    pub email: String,
    pub share_class: String,
}

impl CompanyMetadata {
    pub fn new(name: &str) -> CompanyMetadata {
        CompanyMetadata {
            name: name.to_string(),
            country: "CH".to_string(),
            tags: vec![],
            comment: "".to_string(),
            links: vec![],
            annual_closing_date: "31.12".to_string(),
            accounting_rules: "IFRS".to_string(),
            legal_form: "AG".to_string(),
            url: "".to_string(),
            email: "".to_string(),
            share_class: "RS".to_string(),
        }
    }

    pub fn from_metadata(filename: &str) -> CompanyMetadata {
        let contents =
            &fs::read(&filename).unwrap_or_else(|_| panic!("Reading file {} failed", &filename));
        let metadata_json: String = String::from_utf8_lossy(contents)
            .parse()
            .expect("failed converting to string");
        serde_json::from_str(&metadata_json).unwrap()
    }

    pub fn get_domainname(&self, remove_www: bool) -> String {
        let domainname = self.url
            .replace("https://", "")
            .replace("http://", "");
        if remove_www {
            domainname.replace("www.", "")
        } else {
            domainname
        }
    }
}

pub fn get_metadata<F>(path: &str, f: F) -> Vec<CompanyMetadata>
where
    F: Fn(&CompanyMetadata) -> bool,
{
    let paths = fs::read_dir(path).unwrap();
    let mut metas = Vec::new();
    for source_file in paths {
        let path = source_file.unwrap().path();
        let meta = CompanyMetadata::from_metadata(path.to_str().unwrap());
        if f(&meta) {
            metas.push(meta);
        }
    }
    metas.sort();
    metas
}

#[derive(Debug, Deserialize)]
pub struct Company {
    pub metadata: CompanyMetadata,
    pub reports: Vec<Report>,
    pub oldest_year: u16,
    pub newest_year: u16,
}

impl Company {
    pub fn new(reports: Vec<Report>) -> Company {
        let name = if !reports.is_empty() {
            reports[0].company.to_owned()
        } else {
            String::new()
        };
        let newest_year = reports.iter().fold(0, |acc, x| std::cmp::max(acc, x.year));
        let oldest_year = reports
            .iter()
            .fold(u16::MAX, |acc, x| std::cmp::min(acc, x.year));

        let filename = format!("metadata/{}.json", &name);
        let metadata = if Path::new(&filename).exists() {
            CompanyMetadata::from_metadata(&filename)
        } else {
            CompanyMetadata::new(&name)
        };
        write_metadata(&metadata);

        Company {
            metadata,
            reports,
            oldest_year,
            newest_year,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Download {
    pub report: Report,
    pub size: u64,
    pub mime_type: String,
}

impl Download {
    pub fn has_warning(&self) -> bool {
        self.mime_type != "application/pdf" || self.size < 10
    }
}

pub struct CompanyDownloads {
    pub company: Company,
    pub downloads: Vec<Download>,
}

impl CompanyDownloads {
    pub fn get_number_warnings(&self) -> usize {
        self.downloads.iter().filter(|&d| d.has_warning()).count()
    }

    pub fn get_reports(&self, year: u16, language: &str) -> Vec<&Download> {
        let iter = self
            .downloads
            .iter()
            .filter(|d| d.report.year == year && d.report.language == language);
        iter.collect()
    }
}

pub fn filter_companies<'a>(
    tag: &str,
    companies: &'a [CompanyDownloads],
) -> Vec<&'a CompanyDownloads> {
    companies
        .iter()
        .filter(|c| c.company.metadata.tags.iter().any(|e| e == tag))
        .collect()
}
