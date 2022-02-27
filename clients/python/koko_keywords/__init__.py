import os
from cffi import FFI

ffi = FFI()
current_dir = os.path.dirname(os.path.abspath(__file__))

ffi.cdef("""
int koko_keywords_match(const char *input, const char *filter, const char *version);
""")

lib = ffi.dlopen(current_dir + "/libkoko.dylib")

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
