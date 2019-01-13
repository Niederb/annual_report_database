use std::error::Error;
use std::io::copy;
use std::fs;
use std::fs::File;
use std::path::Path;
use clap::{Arg, App};

use serde_derive::Deserialize;

#[derive(Debug,Deserialize)]
enum CompanyType {
    Smi, // Swiss market index
    Sli, // Swiss leader index
    SmiMid,
    Public,
    Federal,
    Canton,
    Cooperative,
    Infrastructure,
}

#[derive(Debug,Deserialize)]
struct Company {
    company: String,
    company_type: CompanyType,
}

#[derive(Debug,Deserialize)]
struct Report {
    company: String,
    language: String,
    report_type: String,
    year: u16,
    link: String
}

fn download(root_path: &Path, report: &Report) -> Result<(), Box<Error>> {
    let fname = format!("{}-{}.pdf", report.report_type, report.language);
    
    let path = root_path.join(&report.company);
    let path = path.join(&report.year.to_string());
    fs::create_dir_all(&path)?;
    let fname = path.join(fname);
    if !fname.exists() {
        println!("will be located under: '{:?}'", fname);
        let mut dest = File::create(fname)?;
        let mut response = reqwest::get(&report.link)?;
        copy(&mut response, &mut dest)?;
    }
    Ok(())
}

fn iterate_files(root_path: &Path, file: &File) -> Result<(), Box<Error>> {
    let mut rdr = csv::ReaderBuilder::new().delimiter(b';').from_reader(file);
    for result in rdr.deserialize() {
        let report: Report = result?;
        download(&root_path, &report);
        println!("{:?}", report);
    }
    Ok(())
}

fn main() -> Result<(), Box<Error>> {
    let matches = App::new("My Super Program")
        .version("1.0")
        .author("Thomas Niederberger <thomas@niederberger.com>")
        .about("Does awesome things")
        .arg(Arg::with_name("download-directory")
            .short("d")
            .help("Directory into which to download the files")
            .takes_value(true))
        .arg(Arg::with_name("source-directory")
            .short("s")
            .help("Directory that contains the data sources")
            .takes_value(true))
        .get_matches();

    let root_path = Path::new(matches.value_of("download-directory").unwrap_or("C:/Repos/Rust/downloader/downloads"));
    let source_path = Path::new(matches.value_of("source-directory").unwrap_or("C:/Users/Astrid/Dropbox/Actares/Sources"));
    
    let paths = fs::read_dir(source_path).unwrap();

    for source_file in paths {
        let source_file = source_file.unwrap();
        println!("Processing: {}", source_file.path().display());
        //let path = source_path.join("Geberit.csv");
        let file = File::open(source_file.path())?;

        iterate_files(&root_path, &file);
    }
    Ok(())
}