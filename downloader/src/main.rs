use std::error::Error;
use std::io::copy;
use std::fs;
use std::fs::File;
use std::path::Path;
use clap::{Arg, App};

use log::{info, trace, warn, debug, error};

use serde_derive::Deserialize;

#[derive(Debug,Deserialize)]
enum CompanyType {
    Smi, // Swiss market index
    SmiMid,
    Other,
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

fn download(root_path: &Path, report: &Report) -> Result<(), Box<dyn Error>> {
    let fname = format!("{}-{}.pdf", report.report_type, report.language);
    
    let path = root_path.join(&report.company);
    let path = path.join(&report.year.to_string());
    fs::create_dir_all(&path)?;
    let fname = path.join(fname);
    if !fname.exists() {
        debug!("will be located under: '{:?}'", fname);
        let mut dest = File::create(fname)?;
        let mut response = reqwest::get(&report.link)?;
        copy(&mut response, &mut dest)?;
    } else {
        debug!("file already exists: '{:?}'", fname);
    }
    Ok(())
}

fn iterate_files(root_path: &Path, file: &File) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new().delimiter(b';').from_reader(file);
    for result in rdr.deserialize() {
        let report: Report = result?;
        let result = download(&root_path, &report);
        match result {
            Ok(_) => trace!("{:?}", report),
            Err(e) => error!("Error occurred downloading file {}", e),
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    let matches = App::new("Annual report downloader")
        .version("0.1")
        .author("Thomas Niederberger <thomas@niederb.ch>")
        .about("Download annual reports from the Internet")
        .arg(Arg::with_name("download-directory")
            .short("d")
            .help("Directory into which to download the files")
            .takes_value(true))
        .arg(Arg::with_name("source-directory")
            .short("s")
            .help("Directory that contains the data sources")
            .takes_value(true))
        .get_matches();

    let root_path = Path::new(matches.value_of("download-directory").unwrap_or("../downloads"));
    let source_path = Path::new(matches.value_of("source-directory").unwrap_or("../Sources"));
    println!("Downloading into {:?} from source directory {:?}", root_path, source_path);
    let paths = fs::read_dir(source_path).unwrap();

    for source_file in paths {
        let source_file = source_file.unwrap();
        info!("Processing: {}", source_file.path().display());
        let file = File::open(source_file.path())?;

        let result = iterate_files(&root_path, &file);
        match result {
            Ok(_) => (),
            Err(e) => error!("Error deserializing file {:?}", file),
        }
    }
    Ok(())
}