use trust_dns_resolver::config::*;
use trust_dns_resolver::error::ResolveResult;
use trust_dns_resolver::lookup::*;
use trust_dns_resolver::Resolver;
use viaspf_record::Record;

use annual_report_database::data_structures::*;

fn main() {
    let mut resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    let metas = get_metadata("./metadata", |meta| {
        meta.tags.contains(&"Canton".to_string())
    });
    for meta in metas {
        let domainname = meta.get_domainname();
        println!("{:?}, Domainname: {}", meta.name, domainname);
        ipv6_query(&mut resolver, &domainname);
        println!();
    }
}

fn ipv6_query(resolver: &mut Resolver, query: &str) {
    let ipv6_response = resolver.ipv6_lookup(query);

    match ipv6_response {
        Err(_) => println!("No AAAA Records."),
        Ok(ipv6_response) => {
            println!("HAS AAAA Records.");
            for p in ipv6_response.iter() {
                println!("{}", p);
            }
        },
    }
}
