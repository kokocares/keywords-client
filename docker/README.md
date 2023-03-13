# Docker image for running a client in a serverless environment

This folder contains a very think http client for the keywords client library
that makes it easy to use this client in a serverless deployment. Deploy this in
a container with a minimum number of instaces of 1 to ensure that the cache is
maintained in between requests. 

## Image
The image is built and uploaded to
https://hub.docker.io/kokocares/koko_keyword_client

## Setup
There's a single environment variable, `KOKO_KEYWORDS_AUTH`, that must be set to
your `username:password` credentials (similar to using any of the client
libraries.

## Usage
There's a single endpoint availabe, `POST /match` which takes a json object with
two attributes:

* `text`: The text to match against
* `filters`: The keyword filters
  (see https://developers.kokocares.org/docs/overview). The filters is colon delimited list of “dimension=value” filters. Omitting a dimension does not filter by that dimension.

Here's a sample curl request:
```
curl https://keywords-client-server-n2c3m2by7q-uw.a.run.app/match \
  -H 'content-type: application/json' \
  -d '{ "text": "i want to kill myself", "filters": "category=suicide" }'
```

The request returns a json object with a single `matched` attribute:

```json
{ 
  "matched": true
}
```


## CloudRun

To deploy on CloudRun create a new service with the following settings:

* Container image url: kokocares/keyword_client
* Container port: 8080
* CPU allocation and pricing: CPU is only allocated during request processing
* Memory: 512 MiB
* CPU: 1
* Request timeout: 300
* Maximum requests per container: 40
* Execution environment: Default
* Autoscaling: Min instances=1, Max instances=100
* Environment variables: KOKO_KEYWORDS_AUTH=YOUR_CREDENTIALS

You should also setup the service to not be public (the default) and ensure that
it is on the same VPC as your serverless functions.

