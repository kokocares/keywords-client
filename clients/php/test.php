<?php

include 'vendor/autoload.php';

use Koko\Keywords;

$kk = new Keywords();
$match = $kk->match("suicide", "");
var_dump($match);
$match = $kk->match("happy", "");
var_dump($match);
?>
