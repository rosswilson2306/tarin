# Tarin

A tool that automates Lighthouse reports for all relevant URLs on for a given 
list of websites.

## Getting Started

Add a `.env` file to the root of the project and add the following variables:

```
PSI_KEY=XXXXX
PSI_URL=XXXXX
```

At the time of writing the url for the Page Speed Insights api is `https://www.googleapis.com/pagespeedonline/v5/runPagespeed`
but this may change in the future (see [Page Speed Insights documentation](https://developers.google.com/speed/docs/insights/v5/get-started)).

## TODO

- request logging
- test coverage
