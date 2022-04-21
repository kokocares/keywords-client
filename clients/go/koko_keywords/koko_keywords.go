package koko_keywords

/*
  #cgo LDFLAGS: -lkoko_keywords
  #include "libkoko.h"
*/
import "C"

import (
  "errors"
)

func Match(query string, filter string) (bool, error) {
  var match_value int

  match_value = int(C.c_koko_keywords_match(C.CString(query), C.CString(filter)))
  
  if match_value < 0 {
    return false, errors.New(C.GoString(C.c_koko_keywords_error_description(C.long(match_value))))
  }

  switch match_value {
  case 0:
    return false, nil
  case 1:
    return true, nil
  }

  return false, nil
}
