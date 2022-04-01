// TODO: why is import/no-unresolved given here?
// eslint-disable-next-line import/no-unresolved
import test from "ava";

import { match } from "./index.js";

test("match", (t) => {
  t.true(match("sewerslide"));
});

test("match - non match with filter", (t) => {
  t.false(match("sewerslide", "category=wellness"));
});

test("match - non match", (t) => {
  t.false(match("it's all good"));
});

test("match - non match with version", (t) => {
  // TODO move { filters, version } into an object to allow kwargs
  t.false(match("it's all good", undefined, "20220206"));
});
