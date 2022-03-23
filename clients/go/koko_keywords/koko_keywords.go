package koko_keywords

/*
  #cgo LDFLAGS: -lkoko_keywords
  #include "libkoko.h"
*/
import "C"

import (
  "errors"
)

func Match(query string, filter string, version string) (bool, error) {
  var match_value int

  if version == "" {
    match_value = int(C.c_koko_keywords_match(C.CString(query), C.CString(filter), nil))
  } else {
    match_value = int(C.c_koko_keywords_match(C.CString(query), C.CString(filter), C.CString(version)))
  }

  switch match_value {
  case -1:
    return false, errors.New("KOKO_KEYWORDS_AUTH must be set before importing the library")
  case -2:
      return false, errors.New("Invalid credentials. Please confirm you are using valid credentials, contact us at api.kokocares.org if you need assistance.")
  case -3:
      return false, errors.New("Unable to refresh cache. Please try again or contact us at api.kokocares.org if this issue persists.")
  case -4:
      return false, errors.New("Unable to parse response from API. Please contact us at api.kokocares.org if this issue persists.")
  case -5:
      return false, errors.New("Invalid url. Please ensure the url used is valid.")
  case 0:
    return false, nil
  case 1:
    return true, nil
  }

  return false, nil
}
