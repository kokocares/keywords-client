#define FFI_SCOPE "KOKO_KEYWORDS"
#define FFI_LIB "libkoko_keywords.dylib"

int c_koko_keywords_match(const char *input, const char *filter);
const char* c_koko_keywords_error_description(int error);
