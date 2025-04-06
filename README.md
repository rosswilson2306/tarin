# Tarin

A tool that automates Lighthouse reports on all relevant URLs for a given list of websites.

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

## TODO

- gate async tasks to only process 10 sites at any time
- database setup for sites
- post handler to create sites in db
- config for limits on paths of a certain type
- run multiple reports for a single url and take a average
- request logging
- test coverage
