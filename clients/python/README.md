Koko Keyword Python Client
============

A python client  for the [Koko Keywords API](https://r.kokocares.org/koko_keywords/docs). The client handles caching to ensure very low latency.


## Install

```
pip install koko_keywords
```

## Usage

Set the `KOKO_KEYWORDS_AUTH` environment to the authentication string provided
by Koko (contact us at api@kokocares.org if you would like one).

```
export KOKO_KEYWORDS_AUTH=username:password
```

Import the library

```py
import koko_keywords
```

Then use the `match` function to check whether a query prompt matches a risky
keyword

```py
koko_keywords.match("sewerslide")
```

There are two optional params, `filter` and `version`. 

### Filter
Filter the keyword based on the taxonomy using a colon delimited list of “dimension=value” filters. Omitting a dimension does not filter by that dimension e.g.

```py
koko_keywords.match("sewerslide", "category=eating,parenting:confidence=1,2")
```

This matches "sewerslide" against eating eating and parenting, with a confidence of 1 and 2 and any intensity (as intensity was omitted).

### Version
Use this to pin to a specific version of the regex otherwise the endpoint returns the latest.  e.g.

```py
koko_keywords.match("sewerslide", version="20220206")
```

We do not recommend setting this as we frequently update keywords for better matching performance. 

## Performance
The underlying library is written in Rust and cross-compiled to the four major CPU targets. Regexes are cached based on the cache expiration headers (currently set to an hour). This ensures very low latency and overhead (< 1μs/req).


## Error Handling
A RuntimeError will be raised with hints on how to resolve the issue. No exception handling should be necessary.

## Logging
Minimal log messages are logged to STDERR

## License

```
WWWWWW||WWWWWW
 W W W||W W W
      ||
    ( OO )__________
     /  |           \
    /o o|    MIT     \
    \___/||_||__||_|| *
         || ||  || ||
        _||_|| _||_||
       (__|__|(__|__|
```

(The MIT License)

Copyright (c) 2017 Koko AI Inc. <us@kokocares.org>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the 'Software'), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED 'AS IS', WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
