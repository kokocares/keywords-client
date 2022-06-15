Koko Keyword PHP Client
============

A php client  for the [Koko Keywords API](https://developers.kokocares.org). The client handles caching to ensure very low latency.


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
  x86_64](https://github.com/kokocares/keywords-client/releases/download/v0.2.0/libkoko_keywords_x86_64.dylib)
- [OSX,
  ARM64](https://github.com/kokocares/keywords-client/releases/download/v0.2.0/libkoko_keywords_arm64.dylib)
- [Linux,
  x86_64](https://github.com/kokocares/keywords-client/releases/download/v0.2.0/libkoko_keywords_x86_64.dylib)
- [Linux,
  ARM64](https://github.com/kokocares/keywords-client/releases/download/v0.2.0/libkoko_keywords_arm64.dylib)

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

### Install php module

Install the module

```
composer require koko/koko-keywords
```

## Usage

Set the `KOKO_KEYWORDS_AUTH` environment to the authentication string provided
by Koko. To get an api key, complete our [sign up form](https://r.kokocares.org/api_signup).

```
export KOKO_KEYWORDS_AUTH=username:password
```

Import the module

```
```php
include 'vendor/autoload.php';

use Koko\Keywords;

$koko_keywords = new Keywords();
```

It's recommended to instantiate this once to minimize the overhead of loading
the library.

Then use the `match` function to check whether a query prompt matches a risky
keyword. The function returns a `bool` indicating whether there was a match or
not. The function will raise an exception if there is an issue.

```php
if ($koko_keywords.match("some value", "") {
  // Code if there is a match
}

```

There is one optional params, `filter`, set it to the empty
string if you are not using it.

### Filter
Filter the keyword based on the taxonomy using a colon delimited list of “dimension=value” filters. Omitting a dimension does not filter by that dimension e.g.

```php
$koko_keywords.match("sewerslide", "category=eating,parenting:confidence=1,2")
```

This matches "sewerslide" against eating eating and parenting, with a confidence of 1 and 2 and any intensity (as intensity was omitted).

## Performance
The underlying library is written in Rust and cross-compiled to the four major CPU targets. Regexes are cached based on the cache expiration headers (currently set to an hour). This ensures very low latency and overhead (< 1μs/req).


## Error Handling
The `match` function raises an exception if there is an issue.

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
