# Docker image for running a client in a serverless environment

This folder contains a very think http client for the keywords client library
that makes it easy to use this client in a serverless deployment. Deploy this in
a container with a minimum number of instaces of 1 to ensure that the cache is
maintained in between requests. 

## Setup
There's a single environment variable, `KOKO_KEYWORDS_AUTH`, that must be set to
your `username:`password` credentials (similar to using any of the client
libraries.

## Usage
There's a single endpoint availabe, `POST /match` which takes a json object with
two attributes:

* `text`: The text to match against
* `filters`: The keyword filters
  (see https://developers.kokocares.org/docs/overview). The filters is colon delimited list of “dimension=value” filters. Omitting a dimension does not filter by that dimension.

## Quickstart

[![Run on Google Cloud](https://deploy.cloud.run/button.svg)](https://deploy.cloud.run)
