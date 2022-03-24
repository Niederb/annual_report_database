use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use crate::data_structures::CompanyDownloads;

pub fn extract_text(root_dir: &Path, companies: &[CompanyDownloads]) {
    for c in companies {
        for d in &c.downloads {
            
            let mut file_path = d.report.get_file_path(root_dir);
            eprintln!("{:?}", file_path);

            let doc = lopdf::Document::load(&file_path);

            //let contents_result = pdf_extract::extract_text(&file_path);
            file_path.set_extension("txt");
            match doc {
                Ok(doc) => {
                    let total = *doc.get_pages().keys().max().unwrap_or(&0);
                    //let content = doc.extract_text(&[1]);//1..total);
                    let content = doc.extract_text(&(1..total).collect::<Vec<u32>>()[..]);
                    match content {
                        Ok(content) => {
                            let mut file = File::create(&file_path).unwrap();
                            file.write_fmt(format_args!("{}", content)).unwrap();
                        },
                        Err(e) => println!("Error: {}", e),
                    }
                    
                },
                Err(error) => println!("Error: {}", error),
            }
            
            
        }
    }
    
}