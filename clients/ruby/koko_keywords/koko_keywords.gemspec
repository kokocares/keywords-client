
lib = File.expand_path("../lib", __FILE__)
$LOAD_PATH.unshift(lib) unless $LOAD_PATH.include?(lib)
require "koko_keywords/version"

Gem::Specification.new do |spec|
  spec.name          = "koko_keywords"
  spec.version       = KokoKeywords::VERSION
  spec.authors       = ["Kareem Kouddous"]
  spec.email         = ["kkouddous@gmail.com"]

  spec.summary       = "Write a short summary, because RubyGems requires one"
  spec.description   = "Write a longer description or delete this line."
  spec.homepage      = "https://api-docs.kokocares.org"
  spec.license       = "MIT"

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files         = Dir.chdir(File.expand_path('..', __FILE__)) do
    `git ls-files -z`.split("\x0").reject { |f| f.match(%r{^(test|spec|features)/}) }
  end
  spec.bindir        = "exe"
  spec.executables   = spec.files.grep(%r{^exe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  spec.add_dependency("ffi", "~> 1.15")

  spec.add_development_dependency "bundler"
  spec.add_development_dependency "rake"
  spec.add_development_dependency "rspec"
end
