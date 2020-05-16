use horrorshow::helper::doctype;
use horrorshow::{box_html, html, RenderMut};
use std::fs;
use std::fs::File;
use std::io::Write;

use crate::data_structures::{
    get_document_name, get_language, CompanyDownloads, CompanyMetadata, Download,
};

pub fn write_metadata(metadata: &CompanyMetadata) {
    let filename = format!("metadata/{}.json", &metadata.name);
    let serialized = serde_json::to_string_pretty(&metadata).unwrap();

    fs::write(&filename, serialized).expect(&format!("Writing file {} failed", &filename));
}

fn get_disclaimer() -> Box<dyn RenderMut> {
    box_html! {
        p {
            : "Alle Angaben sind ohne Gewähr von Richtigkeit und Vollständigkeit.";
            br;
            : "All information is without guarantee of correctness and completeness.";
            br;
            : "Data and code available on ";
            a (href="https://github.com/Niederb/annual_report_database", target="_blank") {
                : "Github"
            }
            br;
            a (id="warning") {
                : "Warnings: Occur when documents are missing or not pdf files. Typically the reason is that the document was moved or that you need to approve a disclaimer in order to see it"
            }
        }
    }
}

fn get_css_style() -> Box<dyn RenderMut> {
    box_html! {
        style {
            : "table, h1, h2, p, a { font-family:Consolas; }";
            : "table { border-collapse: collapse; width: 100%; }";
            : "td { border: 1px solid black; padding: 5px; }";
        }
    }
}

fn print_reports<'a>(downloads: &'a [&Download]) -> Box<dyn RenderMut + 'a> {
    let target = "_blank";
    box_html! {
        td {
            @ for download in downloads {
                a (href=&download.report.link, target=&target) {
                    @ if download.has_warning() {
                        : format_args!("{} ({} kB, WARNING)", get_document_name(&download.report.report_type), download.size)
                    } else {
                        : format_args!("{} ({} kB)", get_document_name(&download.report.report_type), download.size)
                    }
                }
                br;
            }
        }
    }
}

fn print_sources<'a>(metadata: &'a CompanyMetadata) -> Box<dyn RenderMut + 'a> {
    box_html! {
        @ if !metadata.links.is_empty() {
            h2 {
                : "Sources"
            }
            ul {
                @ for link in &metadata.links {
                    li {
                        a (href=link, target="_blank") {
                            : link
                        }
                    }
                }
            }
        }
    }
}

fn print_html_metadata<'a>(metadata: &'a CompanyMetadata) -> Box<dyn RenderMut + 'a> {
    box_html! {
        meta (name="description", content=format!("Annual reports of {}", metadata.name)) {

        }
        meta (name="robots", content="index, follow") {

        }
        meta (name="keywords", content="annual reports, financial reports, Jahresbericht, Finanzbericht") {

        }
    }
}

pub fn create_reports(companies: &[CompanyDownloads]) {
    // A silly way to convert the slice to a slice of references
    let all_companies = companies.iter().filter(|_| true).collect();
    create_index("html/index.html", &all_companies);
    for company in companies {
        //write_metadata(&company.company.metadata);
        create_company_report(company);
    }
}

pub fn create_index(path: &str, companies: &Vec<&CompanyDownloads>) {

    let (total_documents, total_warnings) = companies.iter().fold((0, 0), |prev, doc| {
        (
            prev.0 + doc.downloads.len(),
            prev.1 + doc.get_number_warnings(),
        )
    });
    let index_content = format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                head {
                    : get_css_style();
                    title : "Annual report database";
                    meta (charset="UTF-8") {

                    }
                }
                body {
                    h1 {
                        : "Annual report database"
                    }
                    p {
                        : format_args!("In total {} documents with {} warnings", total_documents, total_warnings)
                    }
                    table {
                        tr {
                            th {
                                : "Company"
                            }
                            th {
                                : "Origin"
                            }
                            th {
                                : "Annual Closing Date"
                            }
                            th {
                                : "Number documents"
                            }
                            th {
                                : "Data range"
                            }
                            th {
                                : "Warnings";
                                a (href="#warning") {
                                    : "*"
                                }
                            }
                        }
                        @ for company_download in companies {
                            tr {
                                td {
                                    a (href=format_args!("{}.html", company_download.company.metadata.name)) {
                                        : &company_download.company.metadata.name
                                    }
                                }
                                td {
                                    : &company_download.company.metadata.country
                                }
                                td {
                                    : &company_download.company.metadata.annual_closing_date
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
                    : get_disclaimer();
                }
            }
        }
    );
    let mut index_file = File::create(path).unwrap();
    writeln!(index_file, "{}", index_content).unwrap();
}

fn create_company_report(company_download: &CompanyDownloads) {
    let company = &company_download.company;
    
    let metadata = &company_download.company.metadata;
    let company_name = &metadata.name;

    let index_content = format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                head {
                    : get_css_style();
                    : print_html_metadata(&metadata);
                    title : format!("Annual reports of {}", company_name);
                    meta (charset="UTF-8") {

                    }
                }
                body {
                    a (href="index.html") {
                        : "Back"
                    }
                    h1 {
                        @ if metadata.url.is_empty() {
                            : format!("Annual reports of {}", company_name)
                        } else {
                            a (href=&metadata.url, target="_blank") {
                                : format!("Annual reports of {}", company_name)
                            }
                        }
                    }
                    table {
                        tr {
                            th {
                                : "Year"
                            }
                            th {
                                : get_language("EN")
                            }
                            th {
                                : get_language("DE")
                            }
                            th {
                                : get_language("FR")
                            }
                            th {
                                : get_language("IT")
                            }
                        }
                        @ for year in (company.oldest_year..=company.newest_year).rev() {
                            tr {
                                td {
                                    : year
                                }
                                : print_reports(&company_download.get_reports(year, "EN"));
                                : print_reports(&company_download.get_reports(year, "DE"));
                                : print_reports(&company_download.get_reports(year, "FR"));
                                : print_reports(&company_download.get_reports(year, "IT"));
                            }
                        }
                    }
                    : print_sources(metadata);
                    : get_disclaimer();
                    a (href="index.html") {
                        : "Back"
                    }

                }
            }
        }
    );
    let mut index_file = File::create(format!("html/{}.html", &company_name)).unwrap();
    writeln!(index_file, "{}", index_content).unwrap();
}
