
lib = File.expand_path("../lib", __FILE__)
$LOAD_PATH.unshift(lib) unless $LOAD_PATH.include?(lib)
require "koko_keywords/version"

Gem::Specification.new do |spec|
  spec.name          = "koko_keywords"
  spec.version       = KokoKeywords::VERSION
  spec.authors       = ["Kareem Kouddous"]
  spec.email         = ["kkouddous@gmail.com"]

  spec.summary       = "A python client  for the Koko Keywords API. The client handles caching to ensure very low latency."
  spec.description   = File.open('README.md').read
  spec.homepage      = "https://api-docs.kokocares.org"
  spec.license       = "MIT"

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files         = Dir.chdir(File.expand_path('..', __FILE__)) do
    `git ls-files -z`.split("\x0").reject { |f| f.match(%r{^(test|spec|features)/}) }
  end
  spec.require_paths = ["lib"]

  spec.add_dependency "ffi", "~> 1.15"

  spec.add_development_dependency "bundler", "< 2.4"
  spec.add_development_dependency "rake", "< 13.1"
  spec.add_development_dependency "rspec", "< 3.12"
end
