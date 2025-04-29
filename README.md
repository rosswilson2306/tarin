# Tarin

A tool that automates PageSpeed Insights reporting across multiple websites by
generating a tailored list of URLs from sitemaps, providing site-wide performance
reporting to support large-scale web portfolio management.

## Getting Started

Add a `.env` file to the root of the project and add the following variables:

```
PSI_KEY=XXXXX
PSI_URL=XXXXX
SERVER_URL=127.0.0.1:8080
DATABASE_URL=postgres://user:password@127.0.0.1:5432/tarin
```

At the time of writing the url for the Page Speed Insights api is `https://www.googleapis.com/pagespeedonline/v5/runPagespeed`
but this may change in the future (see [Page Speed Insights documentation](https://developers.google.com/speed/docs/insights/v5/get-started)).

### Dependencies

Tarin uses [`sea-orm`](https://www.sea-ql.org/SeaORM/) to manage and interact
with the PostgreQL database. Install `sea-orm-cli`.

```bash
cargo install sea-orm-cli
```

then run

```bash
sea-orm-cli migrate up
```

to apply all pending migrations.

## TODO

- filtering / pagination for get requests
- config for limits on paths of a certain type
- run multiple reports for a single url and take a average
- test coverage
