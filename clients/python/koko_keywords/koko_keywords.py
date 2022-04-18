import os
from cffi import FFI

ffi = FFI()

def find_and_load_dylib():
  ffi.cdef("""
  int c_koko_keywords_match(const char *input, const char *filter);
  const char* c_koko_keywords_error_description(int error);
  """)

  filename = "libkoko_keywords"
  uname = os.uname()
  current_dir = os.path.dirname(os.path.abspath(__file__))

  if os.getenv("KOKO_LIB_PATH"):
      return ffi.dlopen(os.getenv("KOKO_LIB_PATH"))
  elif uname.sysname == 'Darwin' and uname.machine == 'arm64':
      filename = filename + '_arm64.dylib'
  elif uname.sysname == 'Darwin' and uname.machine == 'x86_64':
      filename = filename + '_x86_64.dylib'
  elif uname.sysname == 'Linux' and uname.machine == 'x86_64':
      filename = filename + '_x86_64.so'
  elif uname.sysname == 'Linux' and uname.machine == 'arm64':
      filename = filename + '_arm64.so'
  else:
    raise LookupError(f'Unsupported platform {uname.sysname}, {uname.machine} contact api@kokocares.org for support')

  return ffi.dlopen(current_dir + '/clib/' + filename)

lib = find_and_load_dylib()

def match(text, filters=""):
  match_value = lib.c_koko_keywords_match(text.encode(), filters.encode())

  if match_value < 0:
    raise RuntimeError(str(ffi.string(lib.c_koko_keywords_error_description(match_value)), 'utf-8'))

  return bool(match_value)
