package koko_keywords

import (
  "testing"
  "github.com/kokocares/keywords-client/clients/go/koko_keywords"
  "fmt"
)

func TestMatching(t *testing.T) {
  var tests = []struct {
    query string
    expected bool
  } {
    {"sewerxx   slide", false},
    {"sewerslide", true},
  }

  for _, tt := range tests {
    testname := fmt.Sprintf("%s->%t", tt.query, tt.expected)
    t.Run(testname, func(t *testing.T) {
      res, _ := koko_keywords.Match(tt.query, "", "")
      if res != tt.expected {
        t.Errorf("got %t, expected %t", res, tt.expected)
      }
    })
  }
}
