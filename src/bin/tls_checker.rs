use reqwest::tls::Version;

use annual_report_database::data_structures::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::ClientBuilder::new()
        .use_rustls_tls()
        .min_tls_version(Version::TLS_1_3)
        .build()?;

    let metas = get_metadata("./metadata", |meta| meta.tags.contains(&"SMI".to_string()));
    for meta in metas {
        let domainname = meta.get_domainname(false);
        println!("{:?}, Domainname: {}", meta.name, domainname);
        let res = client.get(meta.url).send().await;
        match res {
            Ok(_) => println!("Success"),
            Err(_) => println!("Error"),
        }
        //println!("{:#?}", res.headers());
    }
    Ok(())
}
