# New Scraper

- Handle patreon and other password based forms
  - Detect whether a chapter is locked behind a form and then post a request to
    get auth cookies (assuming usual wordpress sites)
  - In general, any request that fails to authenticate, or has
    `SkipPasswordProtectedChapters` set should skip indexing and adding to db

- Should be a general scraper.
- Should be able to scrape from any webpage with reasonable page formats, i.e.
  chapters are within one parent div, with titles etc. labeled consistently.
- Should be able to parse from any ToC with a consistent format. I.e. volumes
  contain chapters etc.

- If `RemoveImageEmbeds` is false, images should be downloaded and stored under
  `build/images` with the name format `{chapter_id}_{img_idx}`

## Configuration

- Config should be JSON
- The following should be configurable for each source.
  - ToC format (TODO expand on this)
  - Chapter format (TODO expand on this)
  - Source info (uri, rate limit, ...)
  - output directory, chapter title format, numbering etc.
  - Daemon mode run schedule

## Extra operations

Some additional operations should be configurable for post-scraping.

i.e.

- Epub cover art
  - Provide base cover art, overlay text same as in current version
- Post scrape replacements
  - Regex find and replace strings to be applied in order on the body of the
    scraped chapter. i.e. `r#"<span.*?mrsha-write.*?>(.*?)</span>"#` with
    `"<em>{}</em>"`

## db

Use the same database setup as before

volume (id: P_KEY, name: string, regenerate_epub: bool) chapter (id: P_KEY,
name: string, uri: string, volume: FOREIGN_KEY, data: FOREIGN_KEY,
regenerate_epub: bool) data: (chapter_id: FOREIGN_KEY, data: string)

## Other todo features

- Continue to support auto mail send
- Daemon mode - run on schedule (will not support password scraping password
  protected chapters that require use input)
- Still support sending bundled volumes or individual chapters
  - no need for bundled recent chapters
- use scraper https://docs.rs/scraper/latest/scraper/ instead of soup
- use reqwest https://docs.rs/reqwest/latest/reqwest/
