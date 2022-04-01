use decon_spf::Spf;
use std::fs;
use trust_dns_resolver::config::*;
use trust_dns_resolver::error::ResolveResult;
use trust_dns_resolver::lookup::*;
use trust_dns_resolver::Resolver;
use viaspf_record::Record;

use annual_report_database::data_structures::*;

fn get_metadata() -> Vec<CompanyMetadata> {
    let paths = fs::read_dir("metadata/").unwrap();
    let mut metas = Vec::new();
    for source_file in paths {
        let path = source_file.unwrap().path();
        let meta = CompanyMetadata::from_metadata(path.to_str().unwrap());
        metas.push(meta);
    }
    metas
}

fn main() {
    let mut resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    let metas = get_metadata();
    for meta in metas {
        let url = meta
            .url
            .replace("https://", "")
            .replace("http://", "")
            .replace("www.", "");
        println!("{:?}, URL: {}", meta.name, meta.url);
        spf_query(&mut resolver, &url);
        println!();
    }
}

fn spf_query(resolver: &mut Resolver, query: &str) {
    let txt_response = resolver.txt_lookup(query);
    let spf_record = display_txt(&txt_response);
    println!("Valid: {}, {:?}", spf_record.is_valid(), spf_record);
}

fn display_txt(txt_response: &ResolveResult<TxtLookup>) -> Spf {
    let mut spf_record = Spf::default();
    match txt_response {
        Err(_) => println!("No TXT Records."),
        Ok(txt_response) => {
            let mut i = 1;
            for record in txt_response.iter() {
                if record.to_string().starts_with("v=spf1") {
                    spf_record = record.to_string().parse().unwrap_or(Spf::default());
                    let a = record.to_string().parse::<Record>().unwrap();
                    println!("{}", a);
                }
                i = i + 1;
            }
        }
    }
    spf_record
}
