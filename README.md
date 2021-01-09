# Annual Report Database

A database of annual reports of Swiss companies. I believe it is important that these documents are available for the public in an easy manner such that they can be scrutinized. The more people that are keeping an eye on big companies the better and the easier it is to hold them accountable for their actions. To my knowledge there is no systematic collection of these documents that is available to the public.

My vision is to have a complete and extensive database of reports for all major companies in Switzerland. The database should then allow for different use cases such as:

- Investigating the history of a specific company
- Run statistical analysis

Currently I collect the reports as list of links in csv files. The document are categorized by company, type, language and year. There is also a small software that can be used to download the whole collection in a systematic way.

## What companies?

- Big companies located in Switzerland
- Main focus is on companies listed on the stock exchange (SIX)
  - All companies in the SMI and SMI MID index
  - Companies responsible for major infrastructure (transportation, financial, energy, ...)
  - Companies that are (partially) owned by the public
- Big companies with different other legal forms are not a focus but could also be interesting

## What documents

## What I'm collecting

- Full year reports
  - Annual, financial, governance, sustainability...
- Languages: Official languages of Switzerland plus English

## Currently not collecting but would be interesting

- Minutes from annual meetings
- Financial data in the form of Excel sheets or similar
- Filings to the IRS (internal revenue service)

## What I'm not collecting

- No half year or quaterly reports
- No presentations
- No summaries
- No analyst reports or similar
- No brochures
- Redundant information (for example complete report and separate chapters of the report)

## Known issues

- The UBS report for 2017 is corrupted
- Zurich makes you accept a disclaimer before downloading so automatic download fails for some files
- The Roche reports for 2008 are currently not available anymore
- Swiss RE makes you accept a disclaimer before downloading so automatic download fails for the 2014 report
- BB Biotech also forces you to accept a disclaimer

## Missing companies

## TODO for the downloader

- Sometimes a download can fail
- Compare the newly downloaded pdf to an existing one
- Download only specific companies or years
