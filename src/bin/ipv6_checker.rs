use trust_dns_resolver::config::*;
use trust_dns_resolver::Resolver;

use annual_report_database::data_structures::*;

fn main() {
    let mut resolver = Resolver::new(ResolverConfig::google(), ResolverOpts::default()).unwrap();
    let metas = get_metadata("./metadata", |meta| {
        meta.tags.contains(&"SMI".to_string())
    });
    for meta in metas {
        let domainname = meta.get_domainname(false);
        let has_ipv6_record = ipv6_query(&mut resolver, &domainname);
        println!("{}, {}, {}", meta.name, domainname, has_ipv6_record);
    }
}

fn ipv6_query(resolver: &mut Resolver, query: &str) -> bool {
    let ipv6_response = resolver.ipv6_lookup(query);

    match ipv6_response {
        Err(_) => false,
        Ok(_) => true,
    }
}
