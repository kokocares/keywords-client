FROM ruby:3.2.0

WORKDIR /app
COPY Gemfile /app/Gemfile
COPY Gemfile.lock /app/Gemfile.lock
RUN bundle install
COPY . /app

CMD ["bash", "entrypoint.sh"]
