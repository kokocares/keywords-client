import test from "ava";

import { match } from "../index.js";

test("match", (t) => {
  t.true(match("sewerslide"));
});

test("match - non match with filter", (t) => {
  t.false(match("sewerslide", { filter: "category=wellness" }));
});

test("match - non match", (t) => {
  t.false(match("it's all good"));
});

test("match - non match with version", (t) => {
  t.false(match("it's all good", { version: "20220206" }));
});
