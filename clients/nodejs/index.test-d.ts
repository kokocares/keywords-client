import { expectType } from "tsd";

import { match } from "./index.js";

expectType<Boolean>(match("sewerslide"));
