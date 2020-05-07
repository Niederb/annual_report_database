use horrorshow::helper::doctype;
use horrorshow::{box_html, html, RenderMut};
use std::fs;
use std::fs::File;
use std::io::Write;

use crate::data_structures::{get_document_name, CompanyDownloads, CompanyMetadata, Download};

fn write_metadata(metadata: &CompanyMetadata) {
    let filename = format!("metadata/{}.json", &metadata.name);
    let serialized = serde_json::to_string_pretty(&metadata).unwrap();

    fs::write(&filename, serialized).expect(&format!("Writing file {} failed", &filename));
}

fn get_css_style() -> Box<dyn RenderMut> {
    box_html! {
        style {
            : "table, h1, p, a { font-family:Consolas; }";
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

pub fn create_reports(companies: &[CompanyDownloads]) {
    create_index(companies);
    for company in companies {
        write_metadata(&company.company.metadata);
        create_company_report(company);
    }
}

fn create_index(companies: &[CompanyDownloads]) {
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
                : get_css_style();
                head {
                    title : "Annual reports"
                }
                body {
                    h1 {
                        : "Annual reports"
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
                                : "Number documents"
                            }
                            th {
                                : "Data range"
                            }
                            th {
                                : "Warnings"
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
                }
            }
        }
    );
    let mut index_file = File::create("html/index.html").unwrap();
    writeln!(index_file, "{}", index_content).unwrap();
}

fn create_company_report(company_download: &CompanyDownloads) {
    let company = &company_download.company;
    let company_name = &company_download.company.metadata.name;

    let index_content = format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                : get_css_style();
                head {
                    title : company_name
                }
                body {
                    a (href="index.html") {
                        : "Back"
                    }
                    h1 {
                        : company_name
                    }
                    table {
                        tr {
                            th {
                                : "Year"
                            }
                            th {
                                : "EN"
                            }
                            th {
                                : "DE"
                            }
                            th {
                                : "FR"
                            }
                            th {
                                : "IT"
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
