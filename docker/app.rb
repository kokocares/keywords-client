require 'sinatra/base'
require 'puma'
require 'koko_keywords'

class Keywords < Sinatra::Base
  set :bind, "0.0.0.0"

  before do
    content_type :json
  end

  configure do
    enable :logging
  end

  post '/match' do
    params = JSON.load(request.body)

    if params["text"].nil?
      halt 400, { "error": "text param required" }.to_json
    end

    begin
      result = KokoKeywords.match(params["text"], filters: params["filter"] || '')
      { 'matched': result }.to_json
    rescue => e
      { "error": e.message }.to_json
    end
  end
end
