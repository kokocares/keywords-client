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
    result = KokoKeywords.match(params["text"], filters: params["filter"])

    { 'matched': result }.to_json
  end
end
