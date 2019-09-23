# AnnualReportDatabase
A database of annual reports of Swiss companies. I believe it is important that these documents are available for the public in an easy manner such that they can be scrutinized. The more people are watching

Currently I collect the reports as list of links in csv files. There is also a small download software that can be used to download the whole collection in a systematic way. 

# What companies?
* Big companies located in Switzerland
* Main focus is on companies listed on the stock exchange (SIX)
** All companies in the SMI and SMI MID index
** Companies responsible for major infrastructure (transportation, financial, power, ...)
** Companies that are (partially) owned by the public

# What documents
## What I'm collecting
* Full year reports
** Annual, financial, governance, sustainability...
* Languages: Official languages of Switzerland and English

## Currently not collecting but would be interesting
* Minutes from annual meetings
* Raw data in the form of Excel sheets or similar

## What I'm not collecting
* No half year or quaterly reports
* No presentations
* No summaries
* No analyst reports or similar
* No brochures
* Redundant information (for example complete report and separate chapters of the report)

## Known issues
* UBS has moved many reports
* Sika has moved many reports
* Zurich makes you accept a disclaimer before downloading so automatic download fails for some files
* The Roche reports for 2008 are currently not available anymore
* Swiss RE makes you accept a disclaimer before downloading so automatic download fails for the 2014 report

# TODO for the downloader
* Sometimes a download can fail
* Proper support to check if the download resulted in a valid pdf
* Compare the newly downloaded pdf to an existing one
* Parallelize the downloads
