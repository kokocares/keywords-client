require "koko_keywords/version"
require 'ffi'
require 'etc'

module KokoKeywords
  module NativeWrapper
    def self.lib_path
      uname = Etc.uname

      filename = 'libkoko_keywords'

      if ENV["KOKO_LIB_PATH"]
          return ENV["KOKO_LIB_PATH"]
      elsif uname[:sysname] == 'Darwin' and (uname[:machine] == 'arm64' || uname[:machine] == 'aarch64')
          filename = filename + '_arm64.dylib'
      elsif uname[:sysname] == 'Darwin' and uname[:machine] == 'x86_64'
          filename = filename + '_x86_64.dylib'
      elsif uname[:sysname] == 'Linux' and uname[:machine] == 'x86_64'
          filename = filename + '_x86_64.so'
      elsif uname[:sysname] == 'Linux' and (uname[:machine] == 'arm64' || uname[:machine] == 'aarch64')
          filename = filename + '_arm64.so'
      else
        raise RuntimeError.new("Unsupported platform #{uname[:sysname]}, #{uname[:machine]} contact api@kokocares.org for support")
      end

      __dir__ + '/../clib/' + filename
    end

    extend FFI::Library
    ffi_lib lib_path
    attach_function :c_koko_keywords_match, [:string, :string], :int
    attach_function :c_koko_keywords_error_description, [:int], :string
  end


  def self.match(text, filters: "")
    match_value = NativeWrapper.c_koko_keywords_match(text, filters)

    if match_value < 0
      raise RuntimeError.new(NativeWrapper.c_koko_keywords_error_description(match_value))
    end

    !match_value.zero?
  end
end
