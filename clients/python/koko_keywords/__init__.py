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
  if uname.sysname == 'Darwin' and uname.machine == 'arm64':
      filename = 'libkoko_arm64.dylib'
  elif uname.sysname == 'Darwin' and uname.machine == 'x86_64':
      filename = 'libkoko_x86_64.dylib'
  elif uname.sysname == 'Linux' and uname.machine == 'x86_64':
      filename = 'libkoko_x86_64.so'
  elif uname.sysname == 'Linux' and uname.machine == 'arm64':
      filename = 'libkoko_arm64.so'
  else:
    raise LookupError(f'Unsupported platform {uname.sysname}, {uname.machine} contact api@kokocares.org for support')

  return ffi.dlopen(current_dir + '/' + filename)

lib = find_and_load_dylib()

def main():
  r = lib.koko_keywords_match("sewerslide".encode(), "".encode(), ffi.NULL)
  print(r)
  r = lib.koko_keywords_match("sewerslide".encode(), "category=wellness".encode(), ffi.NULL)
  print(r)
  r = lib.koko_keywords_match("it's all good".encode(), "".encode(), ffi.NULL)
  print(r)
  r = lib.koko_keywords_match("it's all good".encode(), "".encode(), "20220206".encode())
  print(r)
  r = lib.koko_keywords_match("it's all good".encode(), "".encode(), ffi.NULL)
  print(r)

def match(text, filters="", version=None):
  version = version or ffi.NULL
  lib.koko_keywords_match(text.encode(), filters.encode(), version)

if __name__ == "__main__":
  main()
