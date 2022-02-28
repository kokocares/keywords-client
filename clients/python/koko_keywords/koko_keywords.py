import os
from cffi import FFI

ffi = FFI()

def find_and_load_dylib():
  current_dir = os.path.dirname(os.path.abspath(__file__))

  ffi.cdef("""
  int koko_keywords_match(const char *input, const char *filter, const char *version);
  """)

  filname = None
  uname = os.uname()

  if os.getenv("KOKO_LIB_PATH"):
      current_dir = ""
      filename = os.getenv("KOKO_LIB_PATH")
  elif uname.sysname == 'Darwin' and uname.machine == 'arm64':
      filename = 'libkoko_arm64.dylib'
  elif uname.sysname == 'Darwin' and uname.machine == 'x86_64':
      filename = 'libkoko_x86_64.dylib'
  elif uname.sysname == 'Linux' and uname.machine == 'x86_64':
      filename = 'libkoko_x86_64.so'
  elif uname.sysname == 'Linux' and uname.machine == 'arm64':
      filename = 'libkoko_arm64.so'
  else:
    raise LookupError(f'Unsupported platform {uname.sysname}, {uname.machine} contact api@kokocares.org for support')

  return ffi.dlopen(current_dir + '/lib/' + filename)

lib = find_and_load_dylib()

def match(text, filters="", version=None):
  if version:
    version = version.encode()

  match_value = lib.koko_keywords_match(text.encode(), filters.encode(), ffi.NULL)

  if match_value == -1:
    raise RuntimeError("KOKO_KEYWORDS_AUTH or KOKO_KEYWORDS_URL must be set");
  elif match_value == -2:
      raise RuntimeError("Invalid credentials. Please confirm you are using valid credentials, contact us at api.kokocares.org if you need assistance.")
  elif match_value == -3:
      raise RuntimeError("Unable to refresh cache. Please try again or contact us at api.kokocares.org if this issue persists.")
  elif match_value == -4:
      raise RuntimeError("Unable to parse response from API. Please contact us at api.kokocares.org if this issue persists.")

  return match_value
