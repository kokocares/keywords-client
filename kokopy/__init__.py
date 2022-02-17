from cffi import FFI
ffi = FFI()

# cdef() expects a single string declaring the C types, functions and
# globals needed to use the shared object. It must be in valid C syntax.
ffi.cdef("""
bool match_keywords(const char *input, const char *filter);
""")

lib = ffi.dlopen("target/debug/libkoko.dylib")

r = lib.match_keywords("sewerslide".encode(), "".encode())
print(r)
r = lib.match_keywords("sewerslide".encode(), "category=wellness".encode())
print(r)
r = lib.match_keywords("it's all good".encode(), "".encode())
print(r)
