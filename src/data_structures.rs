use serde_derive::Deserialize;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(author, about)]
pub struct Configuration {
    #[structopt(short, long, default_value = "Sources/")]
    pub source_directory: String,

    #[structopt(short, long, default_value = "downloads/")]
    pub download_directory: String,
}

pub fn get_document_name<'a>(abb: &'a str) -> &'a str {
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

#[derive(Debug, Deserialize)]
pub struct Company {
    pub name: String,
    pub reports: Vec<Report>,
    pub oldest_year: u16,
    pub newest_year: u16,
}

impl Company {
    pub fn new(reports: Vec<Report>) -> Company {
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
