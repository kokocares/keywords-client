Koko Keyword Go Client
============

A go client  for the [Koko Keywords API](https://developers.kokocares.org). The client handles caching to ensure very low latency.


## Install

### Install dynamic library

First identify which platform you need to install. Our client currently supports
OSX and Linux across x86_64 and Arm64 architectures. An easy way to do this is
using `uname`:

```
uname -a
```

Download the correct library for your platform:
- [OSX,
  x86_64](https://github.com/kokocares/keywords-client/raw/main/clients/clib/libkoko_keywords_x86_64.dylib)
- [OSX,
  ARM64](https://github.com/kokocares/keywords-client/raw/main/clients/clib/libkoko_keywords_arm64.dylib)
- [Linux,
  x86_64](https://github.com/kokocares/keywords-client/raw/main/clients/clib/libkoko_keywords_x86_64.so)
- [Linux,
  ARM64](https://github.com/kokocares/keywords-client/raw/main/clients/clib/libkoko_keywords_arm64.so)

Move the file to your shared library directory and rename the library removing
the os/architecture label and make sure the file has execution prviledges:

On OSX
```
mv libkoko_keywords_x86_64.dylib /usr/local/lib/libkoko_keywords.dylib
chmod 755 /usr/local/lib/libkoko_keywords.dylib
```

On Linux you can put the file anywhere as you also need to set the LD_LIBRARY_PATH
```
mv libkoko_keywords_x86_64.so /usr/local/lib/libkoko_keywords.so
chmod 755 /usr/local/lib/libkoko_keywords.dylib
export export LD_LIBRARY_PATH=/usr/local/lib
```

### Install go module

Get the module

```
go get github.com/kokocares/keywords-client/clients/go/koko_keywords@v0.1.0
```

## Usage

Set the `KOKO_KEYWORDS_AUTH` environment to the authentication string provided
by Koko. To get an api key, complete our [sign up form](https://r.kokocares.org/api_signup).

```
export KOKO_KEYWORDS_AUTH=username:password
```

Import the module

```
import (
  "github.com/kokocares/keywords-client/clients/go/koko_keywords"
)
```

Then use the `Match` function to check whether a query prompt matches a risky
keyword. The function returns a tuple of type `bool, error`. Error will be `nil`
and the boolean value will represent whether the query matched a keyword or not.

```go
matched, match_err := koko_keywords.Match(tt.query, "", "")

if match_err != nil {
  // Handle error, recommend to log and panic
}

// Do something with matched

```

There are two optional params, `filter` and `version`, set them to the empty
string if you are not using them.

### Filter
Filter the keyword based on the taxonomy using a colon delimited list of “dimension=value” filters. Omitting a dimension does not filter by that dimension e.g.

```go
koko_keywords.Match("sewerslide", "category=eating,parenting:confidence=1,2", "")
```

This matches "sewerslide" against eating eating and parenting, with a confidence of 1 and 2 and any intensity (as intensity was omitted).

### Version
Use this to pin to a specific version of the regex otherwise the endpoint returns the latest. e.g.

```go
koko_keywords.Match("sewerslide", "", "20220206")
```

We do not recommend setting this as we frequently update keywords for better matching performance. 

## Performance
The underlying library is written in Rust and cross-compiled to the four major CPU targets. Regexes are cached based on the cache expiration headers (currently set to an hour). This ensures very low latency and overhead (< 1μs/req).


## Error Handling
The `Match` function returns a tuple `result, error`. If the match has been
performed successfully the error will be returned `nil`, otherwise an error
message will be returned.

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
