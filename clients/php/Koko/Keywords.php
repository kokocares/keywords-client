<?php

namespace Koko;

use FFI;

final class Keywords {
    private static $ffi = null;
    function __construct() {
        if (is_null(self::$ffi)) {
            self::$ffi = FFI::load(__DIR__ . "/../include/koko_keywords.h");
        }
    }
    function match($input, $filter) {
      $result = self::$ffi->c_koko_keywords_match($input, $filter);
      if ($result < 0) {
        throw new Exception(self::$ffi->c_koko_keywords_error_description($result));
      }
      return (int) $result == 1;
    }
}
?>
