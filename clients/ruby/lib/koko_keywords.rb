require "koko_keywords/version"
require 'ffi'
require 'etc'

module KokoKeywords
  module NativeWrapper
    def self.lib_path
      uname = Etc.uname

      if ENV["KOKO_LIB_PATH"]
          return ENV["KOKO_LIB_PATH"]
      elsif uname[:sysname] == 'Darwin' and uname[:machine] == 'arm64'
          filename = 'libkoko_arm64.dylib'
      elsif uname[:sysname] == 'Darwin' and uname[:machine] == 'x86_64'
          filename = 'libkoko_x86_64.dylib'
      elsif uname[:sysname] == 'Linux' and uname[:machine] == 'x86_64'
          filename = 'libkoko_x86_64.so'
      elsif uname[:sysname] == 'Linux' and uname[:machine] == 'arm64'
          filename = 'libkoko_arm64.so'
      else
        raise RuntimeError("Unsupported platform #{uname[:sysname]}, #{uname[:machine]} contact api@kokocares.org for support")
      end

      __dir__ + '/../clib/' + filename
    end

    extend FFI::Library
    ffi_lib lib_path
    attach_function :c_koko_keywords_match, [:string, :string, :string], :int
  end


  def self.match(text, filters: "", version: nil)
    match_value = NativeWrapper.c_koko_keywords_match(text, filters, version)

    if match_value == -1
      raise RuntimeError.new("KOKO_KEYWORDS_AUTH must be set before importing the library")
    elsif match_value == -2
      raise RuntimeError.new("Invalid credentials. Please confirm you are using valid credentials, contact us at api.kokocares.org if you need assistance.")
    elsif match_value == -3
      raise RuntimeError.new("Unable to refresh cache. Please try again or contact us at api.kokocares.org if this issue persists.")
    elsif match_value == -4
      raise RuntimeError.new("Unable to parse response from API. Please contact us at api.kokocares.org if this issue persists.")
    elsif match_value == -5
      raise RuntimeError.new("Invalid url. Please ensure the url used is valid.")
    end

    !match_value.zero?
  end
end
