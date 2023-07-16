import path from "node:path";
import os from "node:os";
import { fileURLToPath } from "node:url";

import ffi from "ffi-napi";

const libraryFilename = (sysname, arch) => {
  if (sysname === "Darwin" && arch === "arm64") {
    return "libkoko_keywords_arm64.dylib";
  } else if (sysname === "Darwin" && arch === "x64") {
    return "libkoko_keywords_x86_64.dylib";
  } else if (sysname === "Linux" && arch === "x64") {
    return "libkoko_keywords_x86_64.so";
  } else if (sysname === "Linux" && arch === "arm64") {
    return "libkoko_keywords_arm64.so";
  } else {
    throw new Error(
      `Unsupported platform ${sysname}, ${arch} contact api@kokocares.org ` +
        "for support"
    );
  }
};

const libraryPath = () => {
  if (process.env.KOKO_LIB_PATH) {
    return process.env.KOKO_LIB_PATH;
  }

  const dirname = path.dirname(fileURLToPath(import.meta.url));
  const filename = libraryFilename(os.type(), os.arch());
  return path.join(dirname, "clib", filename);
};

const loadLibrary = () =>
  // TODO: no need for ext?
  // https://github.com/node-ffi-napi/node-ffi-napi/blob/00df1232a25b1b0f026b5d1b4c9efc67497e4b48/lib/library.js
  ffi.Library(libraryPath(), {
    c_koko_keywords_match: ["int", ["string", "string", "string"]],
  });

const library = loadLibrary();

export const match = (text, filters, version) => {
  const matchValue = library.c_koko_keywords_match(
    text,
    filters === undefined ? "" : filters,
    version === undefined ? null : version
  );
  switch (matchValue) {
    case -1:
      throw new Error(
        "KOKO_KEYWORDS_AUTH must be set before importing the library"
      );
    case -2:
      throw new Error(
        "Invalid credentials. Please confirm you are using valid " +
          "credentials, contact us at api.kokocares.org if you need assistance."
      );
    case -3:
      throw new Error(
        "Unable to refresh cache. Please try again or contact us at " +
          "api.kokocares.org if this issue persists."
      );
    case -4:
      throw new Error(
        "Unable to parse response from API. Please contact us at " +
          "api.kokocares.org if this issue persists."
      );
    case -5:
      throw new Error("Invalid url. Please ensure the url used is valid.");
    default:
      return Boolean(matchValue);
  }
};
