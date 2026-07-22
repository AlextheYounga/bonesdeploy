Source Tree:

```txt
rails
|-- Dockerfile
|-- Gemfile
|-- Gemfile.lock
|-- README.md
|-- Rakefile
|-- app
|   |-- assets
|   |   `-- stylesheets
|   |       `-- application.css
|   |-- controllers
|   |   `-- application_controller.rb
|   |-- helpers
|   |   `-- application_helper.rb
|   |-- jobs
|   |   `-- application_job.rb
|   |-- mailers
|   |   `-- application_mailer.rb
|   |-- models
|   |   `-- application_record.rb
|   `-- views
|       |-- layouts
|       |   |-- application.html.erb
|       |   |-- mailer.html.erb
|       |   `-- mailer.text.erb
|       `-- pwa
|           |-- manifest.json.erb
|           `-- service-worker.js
|-- bin
|   |-- brakeman
|   |-- bundler-audit
|   |-- ci
|   |-- dev
|   |-- docker-entrypoint
|   |-- rails
|   |-- rake
|   |-- rubocop
|   |-- setup
|   `-- thrust
|-- config
|   |-- application.rb
|   |-- boot.rb
|   |-- bundler-audit.yml
|   |-- cable.yml
|   |-- ci.rb
|   |-- credentials.yml.enc
|   |-- database.yml
|   |-- deploy.yml
|   |-- environment.rb
|   |-- environments
|   |   |-- development.rb
|   |   |-- production.rb
|   |   `-- test.rb
|   |-- initializers
|   |   |-- assets.rb
|   |   |-- content_security_policy.rb
|   |   |-- filter_parameter_logging.rb
|   |   `-- inflections.rb
|   |-- locales
|   |   `-- en.yml
|   |-- puma.rb
|   |-- routes.rb
|   `-- storage.yml
|-- config.ru
|-- db
|   `-- seeds.rb
|-- log
|   `-- .keep
|-- public
|   |-- 400.html
|   |-- 404.html
|   |-- 406-unsupported-browser.html
|   |-- 422.html
|   |-- 500.html
|   |-- icon.png
|   |-- icon.svg
|   `-- robots.txt
|-- storage
|   `-- .keep
|-- test
|   `-- test_helper.rb
`-- tmp
    `-- .keep
```

`Dockerfile`:

```txt
# syntax=docker/dockerfile:1
# check=error=true

# This Dockerfile is designed for production, not development. Use with Kamal or build'n'run by hand:
# docker build -t railstest .
# docker run -d -p 80:80 -e RAILS_MASTER_KEY=<value from config/master.key> --name railstest railstest

# For a containerized dev environment, see Dev Containers: https://guides.rubyonrails.org/getting_started_with_devcontainer.html

# Make sure RUBY_VERSION matches the Ruby version in .ruby-version
ARG RUBY_VERSION=3.4.8
FROM docker.io/library/ruby:$RUBY_VERSION-slim AS base

# Rails app lives here
WORKDIR /rails

# Install base packages
RUN apt-get update -qq && \
    apt-get install --no-install-recommends -y curl libjemalloc2 libvips sqlite3 && \
    ln -s /usr/lib/$(uname -m)-linux-gnu/libjemalloc.so.2 /usr/local/lib/libjemalloc.so && \
    rm -rf /var/lib/apt/lists /var/cache/apt/archives

# Set production environment variables and enable jemalloc for reduced memory usage and latency.
ENV RAILS_ENV="production" \
    BUNDLE_DEPLOYMENT="1" \
    BUNDLE_PATH="/usr/local/bundle" \
    BUNDLE_WITHOUT="development" \
    LD_PRELOAD="/usr/local/lib/libjemalloc.so"

# Throw-away build stage to reduce size of final image
FROM base AS build

# Install packages needed to build gems
RUN apt-get update -qq && \
    apt-get install --no-install-recommends -y build-essential git libvips libyaml-dev pkg-config && \
    rm -rf /var/lib/apt/lists /var/cache/apt/archives

# Install application gems
COPY vendor/* ./vendor/
COPY Gemfile Gemfile.lock ./

RUN bundle install && \
    rm -rf ~/.bundle/ "${BUNDLE_PATH}"/ruby/*/cache "${BUNDLE_PATH}"/ruby/*/bundler/gems/*/.git && \
    # -j 1 disable parallel compilation to avoid a QEMU bug: https://github.com/rails/bootsnap/issues/495
    bundle exec bootsnap precompile -j 1 --gemfile

# Copy application code
COPY . .

# Precompile bootsnap code for faster boot times.
# -j 1 disable parallel compilation to avoid a QEMU bug: https://github.com/rails/bootsnap/issues/495
RUN bundle exec bootsnap precompile -j 1 app/ lib/

# Precompiling assets for production without requiring secret RAILS_MASTER_KEY
RUN SECRET_KEY_BASE_DUMMY=1 ./bin/rails assets:precompile




# Final stage for app image
FROM base

# Run and own only the runtime files as a non-root user for security
RUN groupadd --system --gid 1000 rails && \
    useradd rails --uid 1000 --gid 1000 --create-home --shell /bin/bash
USER 1000:1000

# Copy built artifacts: gems, application
COPY --chown=rails:rails --from=build "${BUNDLE_PATH}" "${BUNDLE_PATH}"
COPY --chown=rails:rails --from=build /rails /rails

# Entrypoint prepares the database.
ENTRYPOINT ["/rails/bin/docker-entrypoint"]

# Start server via Thruster by default, this can be overwritten at runtime
EXPOSE 80
CMD ["./bin/thrust", "./bin/rails", "server"]
```

`Gemfile`:

```txt
source "https://rubygems.org"

# Bundle edge Rails instead: gem "rails", github: "rails/rails", branch: "main"
gem "rails", "~> 8.1.3"
# The modern asset pipeline for Rails [https://github.com/rails/propshaft]
gem "propshaft"
# Use sqlite3 as the database for Active Record
gem "sqlite3", ">= 2.1"
# Use the Puma web server [https://github.com/puma/puma]
gem "puma", ">= 5.0"
# Use JavaScript with ESM import maps [https://github.com/rails/importmap-rails]
gem "importmap-rails"
# Hotwire's SPA-like page accelerator [https://turbo.hotwired.dev]
gem "turbo-rails"
# Hotwire's modest JavaScript framework [https://stimulus.hotwired.dev]
gem "stimulus-rails"
# Build JSON APIs with ease [https://github.com/rails/jbuilder]
gem "jbuilder"

# Use Active Model has_secure_password [https://guides.rubyonrails.org/active_model_basics.html#securepassword]
# gem "bcrypt", "~> 3.1.7"

# Windows does not include zoneinfo files, so bundle the tzinfo-data gem
gem "tzinfo-data", platforms: %i[ windows jruby ]

# Use the database-backed adapters for Rails.cache, Active Job, and Action Cable
gem "solid_cache"
gem "solid_queue"
gem "solid_cable"

# Reduces boot times through caching; required in config/boot.rb
gem "bootsnap", require: false

# Deploy this application anywhere as a Docker container [https://kamal-deploy.org]
gem "kamal", require: false

# Add HTTP asset caching/compression and X-Sendfile acceleration to Puma [https://github.com/basecamp/thruster/]
gem "thruster", require: false

# Use Active Storage variants [https://guides.rubyonrails.org/active_storage_overview.html#transforming-images]
gem "image_processing", "~> 1.2"

group :development, :test do
  # See https://guides.rubyonrails.org/debugging_rails_applications.html#debugging-with-the-debug-gem
  gem "debug", platforms: %i[ mri windows ], require: "debug/prelude"

  # Audits gems for known security defects (use config/bundler-audit.yml to ignore issues)
  gem "bundler-audit", require: false

  # Static analysis for security vulnerabilities [https://brakemanscanner.org/]
  gem "brakeman", require: false

  # Omakase Ruby styling [https://github.com/rails/rubocop-rails-omakase/]
  gem "rubocop-rails-omakase", require: false
end

group :development do
  # Use console on exceptions pages [https://github.com/rails/web-console]
  gem "web-console"
end

group :test do
  # Use system testing [https://guides.rubyonrails.org/testing.html#system-testing]
  gem "capybara"
  gem "selenium-webdriver"
end
```

`Gemfile.lock`:

```lock
GEM
  remote: https://rubygems.org/
  specs:
    action_text-trix (2.1.19)
      railties
    actioncable (8.1.3)
      actionpack (= 8.1.3)
      activesupport (= 8.1.3)
      nio4r (~> 2.0)
      websocket-driver (>= 0.6.1)
      zeitwerk (~> 2.6)
    actionmailbox (8.1.3)
      actionpack (= 8.1.3)
      activejob (= 8.1.3)
      activerecord (= 8.1.3)
      activestorage (= 8.1.3)
      activesupport (= 8.1.3)
      mail (>= 2.8.0)
    actionmailer (8.1.3)
      actionpack (= 8.1.3)
      actionview (= 8.1.3)
      activejob (= 8.1.3)
      activesupport (= 8.1.3)
      mail (>= 2.8.0)
      rails-dom-testing (~> 2.2)
    actionpack (8.1.3)
      actionview (= 8.1.3)
      activesupport (= 8.1.3)
      nokogiri (>= 1.8.5)
      rack (>= 2.2.4)
      rack-session (>= 1.0.1)
      rack-test (>= 0.6.3)
      rails-dom-testing (~> 2.2)
      rails-html-sanitizer (~> 1.6)
      useragent (~> 0.16)
    actiontext (8.1.3)
      action_text-trix (~> 2.1.15)
      actionpack (= 8.1.3)
      activerecord (= 8.1.3)
      activestorage (= 8.1.3)
      activesupport (= 8.1.3)
      globalid (>= 0.6.0)
      nokogiri (>= 1.8.5)
    actionview (8.1.3)
      activesupport (= 8.1.3)
      builder (~> 3.1)
      erubi (~> 1.11)
      rails-dom-testing (~> 2.2)
      rails-html-sanitizer (~> 1.6)
    activejob (8.1.3)
      activesupport (= 8.1.3)
      globalid (>= 0.3.6)
    activemodel (8.1.3)
      activesupport (= 8.1.3)
    activerecord (8.1.3)
      activemodel (= 8.1.3)
      activesupport (= 8.1.3)
      timeout (>= 0.4.0)
    activestorage (8.1.3)
      actionpack (= 8.1.3)
      activejob (= 8.1.3)
      activerecord (= 8.1.3)
      activesupport (= 8.1.3)
      marcel (~> 1.0)
    activesupport (8.1.3)
      base64
      bigdecimal
      concurrent-ruby (~> 1.0, >= 1.3.1)
      connection_pool (>= 2.2.5)
      drb
      i18n (>= 1.6, < 2)
      json
      logger (>= 1.4.2)
      minitest (>= 5.1)
      securerandom (>= 0.3)
      tzinfo (~> 2.0, >= 2.0.5)
      uri (>= 0.13.1)
    addressable (2.9.0)
      public_suffix (>= 2.0.2, < 8.0)
    ast (2.4.3)
    base64 (0.3.0)
    bcrypt_pbkdf (1.1.2)
    bigdecimal (4.1.2)
    bindex (0.8.1)
    bootsnap (1.24.6)
      msgpack (~> 1.2)
    brakeman (8.0.5)
      racc
    builder (3.3.0)
    bundler-audit (0.9.3)
      bundler (>= 1.2.0)
      thor (~> 1.0)
    capybara (3.40.0)
      addressable
      matrix
      mini_mime (>= 0.1.3)
      nokogiri (~> 1.11)
      rack (>= 1.6.0)
      rack-test (>= 0.6.3)
      regexp_parser (>= 1.5, < 3.0)
      xpath (~> 3.2)
    concurrent-ruby (1.3.7)
    connection_pool (3.0.2)
    crass (1.0.7)
    date (3.5.1)
    debug (1.11.1)
      irb (~> 1.10)
      reline (>= 0.3.8)
    dotenv (3.2.0)
    drb (2.2.3)
    ed25519 (1.4.0)
    erb (6.0.5)
    erubi (1.13.1)
    et-orbi (1.4.0)
      tzinfo
    ffi (1.17.4-aarch64-linux-gnu)
    ffi (1.17.4-aarch64-linux-musl)
    ffi (1.17.4-arm-linux-gnu)
    ffi (1.17.4-arm-linux-musl)
    ffi (1.17.4-x86_64-linux-gnu)
    ffi (1.17.4-x86_64-linux-musl)
    fugit (1.13.0)
      et-orbi (~> 1.4)
      raabro (~> 1.4)
    globalid (1.4.0)
      activesupport (>= 6.1)
    i18n (1.15.2)
      concurrent-ruby (~> 1.0)
    image_processing (1.14.0)
      mini_magick (>= 4.9.5, < 6)
      ruby-vips (>= 2.0.17, < 3)
    importmap-rails (2.2.3)
      actionpack (>= 6.0.0)
      activesupport (>= 6.0.0)
      railties (>= 6.0.0)
    io-console (0.8.2)
    irb (1.18.0)
      pp (>= 0.6.0)
      prism (>= 1.3.0)
      rdoc (>= 4.0.0)
      reline (>= 0.4.2)
    jbuilder (2.15.1)
      actionview (>= 7.0.0)
      activesupport (>= 7.0.0)
    json (2.21.1)
    kamal (2.12.0)
      activesupport (>= 7.0)
      base64 (~> 0.2)
      bcrypt_pbkdf (~> 1.0)
      concurrent-ruby (~> 1.2)
      dotenv (~> 3.1)
      ed25519 (~> 1.4)
      net-ssh (~> 7.3)
      sshkit (>= 1.23.0, < 2.0)
      thor (~> 1.3)
      zeitwerk (>= 2.6.18, < 3.0)
    language_server-protocol (3.17.0.6)
    lint_roller (1.1.0)
    logger (1.7.0)
    loofah (2.25.2)
      crass (~> 1.0.2)
      nokogiri (>= 1.12.0)
    mail (2.9.1)
      logger
      mini_mime (>= 0.1.1)
      net-imap
      net-pop
      net-smtp
    marcel (1.2.1)
    matrix (0.4.3)
    mini_magick (5.3.2)
      logger
    mini_mime (1.1.5)
    minitest (6.0.6)
      drb (~> 2.0)
      prism (~> 1.5)
    msgpack (1.8.3)
    net-imap (0.6.4.1)
      date
      net-protocol
    net-pop (0.1.2)
      net-protocol
    net-protocol (0.2.2)
      timeout
    net-scp (4.1.0)
      net-ssh (>= 2.6.5, < 8.0.0)
    net-sftp (4.0.0)
      net-ssh (>= 5.0.0, < 8.0.0)
    net-smtp (0.5.1)
      net-protocol
    net-ssh (7.3.3)
    nio4r (2.7.5)
    nokogiri (1.19.4-aarch64-linux-gnu)
      racc (~> 1.4)
    nokogiri (1.19.4-aarch64-linux-musl)
      racc (~> 1.4)
    nokogiri (1.19.4-arm-linux-gnu)
      racc (~> 1.4)
    nokogiri (1.19.4-arm-linux-musl)
      racc (~> 1.4)
    nokogiri (1.19.4-x86_64-linux-gnu)
      racc (~> 1.4)
    nokogiri (1.19.4-x86_64-linux-musl)
      racc (~> 1.4)
    ostruct (0.6.3)
    parallel (2.1.0)
    parser (3.3.12.0)
      ast (~> 2.4.1)
      racc
    pp (0.6.4)
      prettyprint
    prettyprint (0.2.0)
    prism (1.9.0)
    propshaft (1.3.2)
      actionpack (>= 7.0.0)
      activesupport (>= 7.0.0)
      rack
    public_suffix (7.0.5)
    puma (8.0.2)
      nio4r (~> 2.0)
    raabro (1.4.0)
    racc (1.8.1)
    rack (3.2.6)
    rack-session (2.1.2)
      base64 (>= 0.1.0)
      rack (>= 3.0.0)
    rack-test (2.2.0)
      rack (>= 1.3)
    rackup (2.3.1)
      rack (>= 3)
    rails (8.1.3)
      actioncable (= 8.1.3)
      actionmailbox (= 8.1.3)
      actionmailer (= 8.1.3)
      actionpack (= 8.1.3)
      actiontext (= 8.1.3)
      actionview (= 8.1.3)
      activejob (= 8.1.3)
      activemodel (= 8.1.3)
      activerecord (= 8.1.3)
      activestorage (= 8.1.3)
      activesupport (= 8.1.3)
      bundler (>= 1.15.0)
      railties (= 8.1.3)
    rails-dom-testing (2.3.0)
      activesupport (>= 5.0.0)
      minitest
      nokogiri (>= 1.6)
    rails-html-sanitizer (1.7.1)
      loofah (~> 2.25, >= 2.25.2)
      nokogiri (>= 1.15.7, != 1.16.7, != 1.16.6, != 1.16.5, != 1.16.4, != 1.16.3, != 1.16.2, != 1.16.1, != 1.16.0.rc1, != 1.16.0)
    railties (8.1.3)
      actionpack (= 8.1.3)
      activesupport (= 8.1.3)
      irb (~> 1.13)
      rackup (>= 1.0.0)
      rake (>= 12.2)
      thor (~> 1.0, >= 1.2.2)
      tsort (>= 0.2)
      zeitwerk (~> 2.6)
    rainbow (3.1.1)
    rake (13.4.2)
    rbs (4.0.3)
      logger
      prism (>= 1.6.0)
      tsort
    rdoc (8.0.0)
      erb
      prism (>= 1.6.0)
      rbs (>= 4.0.0)
      tsort
    regexp_parser (2.12.0)
    reline (0.6.3)
      io-console (~> 0.5)
    rexml (3.4.4)
    rubocop (1.88.2)
      json (~> 2.3)
      language_server-protocol (~> 3.17.0.2)
      lint_roller (~> 1.1.0)
      parallel (>= 1.10)
      parser (>= 3.3.0.2)
      rainbow (>= 2.2.2, < 4.0)
      regexp_parser (>= 2.9.3, < 3.0)
      rubocop-ast (>= 1.49.0, < 2.0)
      ruby-progressbar (~> 1.7)
      unicode-display_width (>= 2.4.0, < 4.0)
    rubocop-ast (1.50.0)
      parser (>= 3.3.7.2)
      prism (~> 1.7)
    rubocop-performance (1.26.1)
      lint_roller (~> 1.1)
      rubocop (>= 1.75.0, < 2.0)
      rubocop-ast (>= 1.47.1, < 2.0)
    rubocop-rails (2.36.0)
      activesupport (>= 4.2.0)
      lint_roller (~> 1.1)
      rack (>= 1.1)
      rubocop (>= 1.75.0, < 2.0)
      rubocop-ast (>= 1.44.0, < 2.0)
    rubocop-rails-omakase (1.1.0)
      rubocop (>= 1.72)
      rubocop-performance (>= 1.24)
      rubocop-rails (>= 2.30)
    ruby-progressbar (1.13.0)
    ruby-vips (2.3.0)
      ffi (~> 1.12)
      logger
    rubyzip (3.4.1)
    securerandom (0.4.1)
    selenium-webdriver (4.46.0)
      base64 (~> 0.2)
      logger (~> 1.4)
      rexml (~> 3.2, >= 3.2.5)
      rubyzip (>= 1.2.2, < 4.0)
      websocket (~> 1.0)
    solid_cable (4.0.0)
      actioncable (>= 7.2)
      activejob (>= 7.2)
      activerecord (>= 7.2)
      railties (>= 7.2)
    solid_cache (1.0.10)
      activejob (>= 7.2)
      activerecord (>= 7.2)
      railties (>= 7.2)
    solid_queue (1.4.0)
      activejob (>= 7.1)
      activerecord (>= 7.1)
      concurrent-ruby (>= 1.3.1)
      fugit (~> 1.11)
      railties (>= 7.1)
      thor (>= 1.3.1)
    sqlite3 (2.9.5-aarch64-linux-gnu)
    sqlite3 (2.9.5-aarch64-linux-musl)
    sqlite3 (2.9.5-arm-linux-gnu)
    sqlite3 (2.9.5-arm-linux-musl)
    sqlite3 (2.9.5-x86_64-linux-gnu)
    sqlite3 (2.9.5-x86_64-linux-musl)
    sshkit (1.25.0)
      base64
      logger
      net-scp (>= 1.1.2)
      net-sftp (>= 2.1.2)
      net-ssh (>= 2.8.0)
      ostruct
    stimulus-rails (1.3.4)
      railties (>= 6.0.0)
    thor (1.5.0)
    thruster (0.1.23)
    thruster (0.1.23-aarch64-linux)
    thruster (0.1.23-x86_64-linux)
    timeout (0.6.1)
    tsort (0.2.0)
    turbo-rails (2.0.23)
      actionpack (>= 7.1.0)
      railties (>= 7.1.0)
    tzinfo (2.0.6)
      concurrent-ruby (~> 1.0)
    unicode-display_width (3.2.0)
      unicode-emoji (~> 4.1)
    unicode-emoji (4.2.0)
    uri (1.1.1)
    useragent (0.16.11)
    web-console (4.3.0)
      actionview (>= 8.0.0)
      bindex (>= 0.4.0)
      railties (>= 8.0.0)
    websocket (1.2.11)
    websocket-driver (0.8.2)
      base64
      websocket-extensions (>= 0.1.0)
    websocket-extensions (0.1.5)
    xpath (3.2.0)
      nokogiri (~> 1.8)
    zeitwerk (2.8.2)

PLATFORMS
  aarch64-linux
  aarch64-linux-gnu
  aarch64-linux-musl
  arm-linux-gnu
  arm-linux-musl
  x86_64-linux-gnu
  x86_64-linux-musl

DEPENDENCIES
  bootsnap
  brakeman
  bundler-audit
  capybara
  debug
  image_processing (~> 1.2)
  importmap-rails
  jbuilder
  kamal
  propshaft
  puma (>= 5.0)
  rails (~> 8.1.3)
  rubocop-rails-omakase
  selenium-webdriver
  solid_cable
  solid_cache
  solid_queue
  sqlite3 (>= 2.1)
  stimulus-rails
  thruster
  turbo-rails
  tzinfo-data
  web-console

CHECKSUMS
  action_text-trix (2.1.19) sha256=7012f59421009cf284aa651294896414d653a61a2417c9b8714c8476d2f74009
  actioncable (8.1.3) sha256=e5bc7f75e44e6a22de29c4f43176927c3a9ce4824464b74ed18d8226e75a80f0
  actionmailbox (8.1.3) sha256=df7da474eaa0e70df4ed5a6fef66eb3b3b0f2dbf7f14518deee8d77f1b4aae59
  actionmailer (8.1.3) sha256=831f724891bb70d0aaa4d76581a6321124b6a752cb655c9346aae5479318448d
  actionpack (8.1.3) sha256=af998cae4d47c5d581a2cc363b5c77eb718b7c4b45748d81b1887b25621c29a3
  actiontext (8.1.3) sha256=d291019c00e1ea9e6463011fa214f6081a56d7b9a1d224e7d3f6384c1dafc7d2
  actionview (8.1.3) sha256=1347c88c7f3edb38100c5ce0e9fb5e62d7755f3edc1b61cce2eb0b2c6ea2fd5d
  activejob (8.1.3) sha256=a149b1766aa8204c3c3da7309e4becd40fcd5529c348cffbf6c9b16b565fe8d3
  activemodel (8.1.3) sha256=90c05cbe4cef3649b8f79f13016191ea94c4525ce4a5c0fb7ef909c4b91c8219
  activerecord (8.1.3) sha256=8003be7b2466ba0a2a670e603eeb0a61dd66058fccecfc49901e775260ac70ab
  activestorage (8.1.3) sha256=0564ce9309143951a67615e1bb4e090ee54b8befed417133cae614479b46384d
  activesupport (8.1.3) sha256=21a5e0dfbd4c3ddd9e1317ec6a4d782fa226e7867dc70b0743acda81a1dca20e
  addressable (2.9.0) sha256=7fdf6ac3660f7f4e867a0838be3f6cf722ace541dd97767fa42bc6cfa980c7af
  ast (2.4.3) sha256=954615157c1d6a382bc27d690d973195e79db7f55e9765ac7c481c60bdb4d383
  base64 (0.3.0) sha256=27337aeabad6ffae05c265c450490628ef3ebd4b67be58257393227588f5a97b
  bcrypt_pbkdf (1.1.2) sha256=c2414c23ce66869b3eb9f643d6a3374d8322dfb5078125c82792304c10b94cf6
  bigdecimal (4.1.2) sha256=53d217666027eab4280346fba98e7d5b66baaae1b9c3c1c0ffe89d48188a3fbd
  bindex (0.8.1) sha256=7b1ecc9dc539ed8bccfc8cb4d2732046227b09d6f37582ff12e50a5047ceb17e
  bootsnap (1.24.6) sha256=c60bab88c70332290f0a2636a288f675299eb4f804a02a3c085b42eca9da164a
  brakeman (8.0.5) sha256=03735f9690d3fd4b32d66aacbf0a6d15a84266bdd06b32c05c8ecc8f6021d2be
  builder (3.3.0) sha256=497918d2f9dca528fdca4b88d84e4ef4387256d984b8154e9d5d3fe5a9c8835f
  bundler-audit (0.9.3) sha256=81c8766c71e47d0d28a0f98c7eed028539f21a6ea3cd8f685eb6f42333c9b4e9
  capybara (3.40.0) sha256=42dba720578ea1ca65fd7a41d163dd368502c191804558f6e0f71b391054aeef
  concurrent-ruby (1.3.7) sha256=4412caec3a5ea2e5fdc52076724c071a81f2c0593d83b2ac8cbb8ca63b3151b0
  connection_pool (3.0.2) sha256=33fff5ba71a12d2aa26cb72b1db8bba2a1a01823559fb01d29eb74c286e62e0a
  crass (1.0.7) sha256=94868719948664c89ddcaf0a37c65048413dfcb1c869470a5f7a7ceb5390b295
  date (3.5.1) sha256=750d06384d7b9c15d562c76291407d89e368dda4d4fff957eb94962d325a0dc0
  debug (1.11.1) sha256=2e0b0ac6119f2207a6f8ac7d4a73ca8eb4e440f64da0a3136c30343146e952b6
  dotenv (3.2.0) sha256=e375b83121ea7ca4ce20f214740076129ab8514cd81378161f11c03853fe619d
  drb (2.2.3) sha256=0b00d6fdb50995fe4a45dea13663493c841112e4068656854646f418fda13373
  ed25519 (1.4.0) sha256=16e97f5198689a154247169f3453ef4cfd3f7a47481fde0ae33206cdfdcac506
  erb (6.0.5) sha256=858e63488cb796c9daba8b6e9ff4b3879c395022049be9a66a8e00980e612eac
  erubi (1.13.1) sha256=a082103b0885dbc5ecf1172fede897f9ebdb745a4b97a5e8dc63953db1ee4ad9
  et-orbi (1.4.0) sha256=6c7e3c90779821f9e3b324c5e96fda9767f72995d6ae435b96678a4f3e2de8bc
  ffi (1.17.4-aarch64-linux-gnu) sha256=b208f06f91ffd8f5e1193da3cae3d2ccfc27fc36fba577baf698d26d91c080df
  ffi (1.17.4-aarch64-linux-musl) sha256=9286b7a615f2676245283aef0a0a3b475ae3aae2bb5448baace630bb77b91f39
  ffi (1.17.4-arm-linux-gnu) sha256=d6dbddf7cb77bf955411af5f187a65b8cd378cb003c15c05697f5feee1cb1564
  ffi (1.17.4-arm-linux-musl) sha256=9d4838ded0465bef6e2426935f6bcc93134b6616785a84ffd2a3d82bc3cf6f95
  ffi (1.17.4-x86_64-linux-gnu) sha256=9d3db14c2eae074b382fa9c083fe95aec6e0a1451da249eab096c34002bc752d
  ffi (1.17.4-x86_64-linux-musl) sha256=3fdf9888483de005f8ef8d1cf2d3b20d86626af206cbf780f6a6a12439a9c49e
  fugit (1.13.0) sha256=a4f093fce740da52f216740a5041e2a594ea763cdb89e8b2754ca4399634ab18
  globalid (1.4.0) sha256=037f12fbf1d9d7a014d501c2d5c77356fd4ddd96d7a7991d6700bba96706f427
  i18n (1.15.2) sha256=00f9eb62412fe593b2a65a97daa75300d37abb8f7202ec748e94b6d46a9dd1b5
  image_processing (1.14.0) sha256=754cc169c9c262980889bec6bfd325ed1dafad34f85242b5a07b60af004742fb
  importmap-rails (2.2.3) sha256=7101be2a4dc97cf1558fb8f573a718404c5f6bcfe94f304bf1f39e444feeb16a
  io-console (0.8.2) sha256=d6e3ae7a7cc7574f4b8893b4fca2162e57a825b223a177b7afa236c5ef9814cc
  irb (1.18.0) sha256=de9454a0703a54704b9811a5ef31a60c86949fbf4013fcf244fabc7c775248e3
  jbuilder (2.15.1) sha256=2430bec28fb0cebacb5875b1009cf9d8bc3c303ccb810c4c8b062a4b51457637
  json (2.21.1) sha256=13a43df75d95641443f5702dff350f237164a9d811ff0f2c2800d4d980220583
  kamal (2.12.0) sha256=c51d1ab085e515470f98d0c0f043637122b5ebf76e8b610cb1fbbed0b7f9b8fa
  language_server-protocol (3.17.0.6) sha256=5ef2c0c138f8267e1bc631d3328347d354f96724b0af22f2c79516120443b7f0
  lint_roller (1.1.0) sha256=2c0c845b632a7d172cb849cc90c1bce937a28c5c8ccccb50dfd46a485003cc87
  logger (1.7.0) sha256=196edec7cc44b66cfb40f9755ce11b392f21f7967696af15d274dde7edff0203
  loofah (2.25.2) sha256=2007f746959ac65552456e04b433e83deb22759ab38c838b4445c70e43425918
  mail (2.9.1) sha256=06574eca475253d6c18145dd70af80d0eb970182d55053497c5f4d797ea160e8
  marcel (1.2.1) sha256=1678e9360e32f9eafa917c80029e2f6d10b2715c66a4b87b6d0da9b9cd1f859f
  matrix (0.4.3) sha256=a0d5ab7ddcc1973ff690ab361b67f359acbb16958d1dc072b8b956a286564c5b
  mini_magick (5.3.2) sha256=a7ef54b480b15283c396eb8343d9e088471f54b31d9974cb7e523cc57598609f
  mini_mime (1.1.5) sha256=8681b7e2e4215f2a159f9400b5816d85e9d8c6c6b491e96a12797e798f8bccef
  minitest (6.0.6) sha256=153ea36d1d987a62942382b61075745042a2b3123b1cd48f4c3675af9cc7d6f1
  msgpack (1.8.3) sha256=8bda4a6428d3244e50d6bd55854d354edbada88a4e1f4f5731a39a0f86bee6a1
  net-imap (0.6.4.1) sha256=29f0360d75a7efd3539f16ac1957dea5c0a51ddeceb348db4553c3120914ea0d
  net-pop (0.1.2) sha256=848b4e982013c15b2f0382792268763b748cce91c9e91e36b0f27ed26420dff3
  net-protocol (0.2.2) sha256=aa73e0cba6a125369de9837b8d8ef82a61849360eba0521900e2c3713aa162a8
  net-scp (4.1.0) sha256=a99b0b92a1e5d360b0de4ffbf2dc0c91531502d3d4f56c28b0139a7c093d1a5d
  net-sftp (4.0.0) sha256=65bb91c859c2f93b09826757af11b69af931a3a9155050f50d1b06d384526364
  net-smtp (0.5.1) sha256=ed96a0af63c524fceb4b29b0d352195c30d82dd916a42f03c62a3a70e5b70736
  net-ssh (7.3.3) sha256=831def58b2c51dcef66ec00d29397d4f210de89c19fe78f95873ca30f386e86a
  nio4r (2.7.5) sha256=6c90168e48fb5f8e768419c93abb94ba2b892a1d0602cb06eef16d8b7df1dca1
  nokogiri (1.19.4-aarch64-linux-gnu) sha256=1269fb644a6de405057a53dd5c762b1209b43ca7424f839454d3dbc677c31a8f
  nokogiri (1.19.4-aarch64-linux-musl) sha256=35c65b9ce72b3bb03207bdbe7067915019dc18c1b9b59139684bd6690fdd01af
  nokogiri (1.19.4-arm-linux-gnu) sha256=a301313e38bb065d68239e79734bcd6f56fb6efaacebde29e9abf2a4735340ca
  nokogiri (1.19.4-arm-linux-musl) sha256=588923c101bcfa78869734d247d25b598674323e7f22474fc468f6e5647311eb
  nokogiri (1.19.4-x86_64-linux-gnu) sha256=379fae440b28915e3f19d752ce2dcf8465ed2b2fbefd2a7ca0dd497bc981a06a
  nokogiri (1.19.4-x86_64-linux-musl) sha256=17dfb7c1fa194ae02fbf7c51a7afc8d278045ab3fdacfd86f91d02d7b274470b
  ostruct (0.6.3) sha256=95a2ed4a4bd1d190784e666b47b2d3f078e4a9efda2fccf18f84ddc6538ed912
  parallel (2.1.0) sha256=b35258865c2e31134c5ecb708beaaf6772adf9d5efae28e93e99260877b09356
  parser (3.3.12.0) sha256=21a6d7f755d5a24dfbdc6e6b772e4e879a52e7631a88bc5a3a134606052c9828
  pp (0.6.4) sha256=dfcb0fce700c41456265922884f9fe195d7fbb0674a3578e6c0f69588e82b570
  prettyprint (0.2.0) sha256=2bc9e15581a94742064a3cc8b0fb9d45aae3d03a1baa6ef80922627a0766f193
  prism (1.9.0) sha256=7b530c6a9f92c24300014919c9dcbc055bf4cdf51ec30aed099b06cd6674ef85
  propshaft (1.3.2) sha256=1d56a3e56a92c21bfc29caf07406b5386b00d4c47ddf357cf989a5a234b1389e
  public_suffix (7.0.5) sha256=1a8bb08f1bbea19228d3bed6e5ed908d1cb4f7c2726d18bd9cadf60bc676f623
  puma (8.0.2) sha256=c8ed871dfbbe66448ea9ffd46692342d9804d4071522b52b5331b7b6e7b686fb
  raabro (1.4.0) sha256=d4fa9ff5172391edb92b242eed8be802d1934b1464061ae5e70d80962c5da882
  racc (1.8.1) sha256=4a7f6929691dbec8b5209a0b373bc2614882b55fc5d2e447a21aaa691303d62f
  rack (3.2.6) sha256=5ed78e1f73b2e25679bec7d45ee2d4483cc4146eb1be0264fc4d94cb5ef212c2
  rack-session (2.1.2) sha256=595434f8c0c3473ae7d7ac56ecda6cc6dfd9d37c0b2b5255330aa1576967ffe8
  rack-test (2.2.0) sha256=005a36692c306ac0b4a9350355ee080fd09ddef1148a5f8b2ac636c720f5c463
  rackup (2.3.1) sha256=6c79c26753778e90983761d677a48937ee3192b3ffef6bc963c0950f94688868
  rails (8.1.3) sha256=6d017ba5348c98fc909753a8169b21d44de14d2a0b92d140d1a966834c3c9cd3
  rails-dom-testing (2.3.0) sha256=8acc7953a7b911ca44588bf08737bc16719f431a1cc3091a292bca7317925c1d
  rails-html-sanitizer (1.7.1) sha256=e797a7c9b01e567307e317c576b49ab4168017e63eea4dba9ce3cb587e2f22c2
  railties (8.1.3) sha256=913eb0e0cb520aac687ffd74916bd726d48fa21f47833c6292576ef6a286de22
  rainbow (3.1.1) sha256=039491aa3a89f42efa1d6dec2fc4e62ede96eb6acd95e52f1ad581182b79bc6a
  rake (13.4.2) sha256=cb825b2bd5f1f8e91ca37bddb4b9aaf345551b4731da62949be002fa89283701
  rbs (4.0.3) sha256=5a7bf70e2628549d9a1f44eae447b2cfe55968a9c60cfff52693a4bdcc020e14
  rdoc (8.0.0) sha256=03bf8c08a9639658855a0cfd77c0abca8325c227693f7f33f82957811348c469
  regexp_parser (2.12.0) sha256=35a916a1d63190ab5c9009457136ae5f3c0c7512d60291d0d1378ba18ce08ebb
  reline (0.6.3) sha256=1198b04973565b36ec0f11542ab3f5cfeeec34823f4e54cebde90968092b1835
  rexml (3.4.4) sha256=19e0a2c3425dfbf2d4fc1189747bdb2f849b6c5e74180401b15734bc97b5d142
  rubocop (1.88.2) sha256=8def251c90cd955feb4daa3edc0ab56893250c4ce90ef81e6c80c03f9a939bbf
  rubocop-ast (1.50.0) sha256=b9ca88300da0803ee222ad20cdb30494c0a784eed06fdc35d254b06d662788db
  rubocop-performance (1.26.1) sha256=cd19b936ff196df85829d264b522fd4f98b6c89ad271fa52744a8c11b8f71834
  rubocop-rails (2.36.0) sha256=105a5f5b0001ff2ef28a3af90da50862464e0775e8d0016d599079c0ca408bdb
  rubocop-rails-omakase (1.1.0) sha256=2af73ac8ee5852de2919abbd2618af9c15c19b512c4cfc1f9a5d3b6ef009109d
  ruby-progressbar (1.13.0) sha256=80fc9c47a9b640d6834e0dc7b3c94c9df37f08cb072b7761e4a71e22cff29b33
  ruby-vips (2.3.0) sha256=e685ec02c13969912debbd98019e50492e12989282da5f37d05f5471442f5374
  rubyzip (3.4.1) sha256=0a79e745b5c25872ebd148457df5665da8530ed8626c993b01457128f173ca02
  securerandom (0.4.1) sha256=cc5193d414a4341b6e225f0cb4446aceca8e50d5e1888743fac16987638ea0b1
  selenium-webdriver (4.46.0) sha256=cb6cfa6f79eac041749df0b14cdcb0e9fe5df3715a64c77bfaa56b345b99f957
  solid_cable (4.0.0) sha256=8379680ef6bf36e195eb876a6306ea290f87d5fa10bc4a757bc2a918f83229b5
  solid_cache (1.0.10) sha256=bc05a2fb3ac78a6f43cbb5946679cf9db67dd30d22939ededc385cb93e120d41
  solid_queue (1.4.0) sha256=e6a18d196f0b27cb6e3c77c5b31258b05fb634f8ed64fb1866ed164047216c2a
  sqlite3 (2.9.5-aarch64-linux-gnu) sha256=78075b6337d3d182c6d2b4691049ed45cd220826160c9ea18946bf6a1de200dc
  sqlite3 (2.9.5-aarch64-linux-musl) sha256=18c801185deb4adc01ddb281e8f672a39e3d1729979ca91e39439cd3eac0402d
  sqlite3 (2.9.5-arm-linux-gnu) sha256=1bdfca0c7d63998c60b0f4a8e3c8df2d33800ccc4abd2d612eddbbbc92a4c48b
  sqlite3 (2.9.5-arm-linux-musl) sha256=bae1109d12b2e9f588455967729b008e1ff4feb7761749df695019c9079913c6
  sqlite3 (2.9.5-x86_64-linux-gnu) sha256=233dbcb6714148dd23bc5aeb33e8efd6eac974969564ddd5794c23d5f52b231e
  sqlite3 (2.9.5-x86_64-linux-musl) sha256=e7d3a7474e8af0f96150c21abc203fbab5437206bfcdf11deab7741c0ca516f2
  sshkit (1.25.0) sha256=c8c6543cdb60f91f1d277306d585dd11b6a064cb44eab0972827e4311ff96744
  stimulus-rails (1.3.4) sha256=765676ffa1f33af64ce026d26b48e8ffb2e0b94e0f50e9119e11d6107d67cb06
  thor (1.5.0) sha256=e3a9e55fe857e44859ce104a84675ab6e8cd59c650a49106a05f55f136425e73
  thruster (0.1.23) sha256=ad3081e8ff3b5cb413768292c367f1e015c423bb6b35919b9b9f533af50d639f
  thruster (0.1.23-aarch64-linux) sha256=042a05581dd1d750d1583d83817460b1f7fde254ed0b2bcd7ec56f9bed278c7a
  thruster (0.1.23-x86_64-linux) sha256=9cd7265bf54e954575c8442e42d050ed8eb55df4f9d7da8fd6f94fcc0339baa5
  timeout (0.6.1) sha256=78f57368a7e7bbadec56971f78a3f5ecbcfb59b7fcbb0a3ed6ddc08a5094accb
  tsort (0.2.0) sha256=9650a793f6859a43b6641671278f79cfead60ac714148aabe4e3f0060480089f
  turbo-rails (2.0.23) sha256=ee0d90733aafff056cf51ff11e803d65e43cae258cc55f6492020ec1f9f9315f
  tzinfo (2.0.6) sha256=8daf828cc77bcf7d63b0e3bdb6caa47e2272dcfaf4fbfe46f8c3a9df087a829b
  unicode-display_width (3.2.0) sha256=0cdd96b5681a5949cdbc2c55e7b420facae74c4aaf9a9815eee1087cb1853c42
  unicode-emoji (4.2.0) sha256=519e69150f75652e40bf736106cfbc8f0f73aa3fb6a65afe62fefa7f80b0f80f
  uri (1.1.1) sha256=379fa58d27ffb1387eaada68c749d1426738bd0f654d812fcc07e7568f5c57c6
  useragent (0.16.11) sha256=700e6413ad4bb954bb63547fa098dddf7b0ebe75b40cc6f93b8d54255b173844
  web-console (4.3.0) sha256=e13b71301cdfc2093f155b5aa3a622db80b4672d1f2f713119cc7ec7ac6a6da4
  websocket (1.2.11) sha256=b7e7a74e2410b5e85c25858b26b3322f29161e300935f70a0e0d3c35e0462737
  websocket-driver (0.8.2) sha256=97c556b019bf3410b4961002ac501621e9322d3f8a7bc02161a09301cc4c4146
  websocket-extensions (0.1.5) sha256=1c6ba63092cda343eb53fc657110c71c754c56484aad42578495227d717a8241
  xpath (3.2.0) sha256=6dfda79d91bb3b949b947ecc5919f042ef2f399b904013eb3ef6d20dd3a4082e
  zeitwerk (2.8.2) sha256=7212a61311083c604184b1ea2574b9aa05cd14f855a0841c06985cabe9181d12

BUNDLED WITH
  4.0.3
```

`README.md`:

```md
# README

This README would normally document whatever steps are necessary to get the
application up and running.

Things you may want to cover:

* Ruby version

* System dependencies

* Configuration

* Database creation

* Database initialization

* How to run the test suite

* Services (job queues, cache servers, search engines, etc.)

* Deployment instructions

* ...
```

`Rakefile`:

```txt
# Add your own tasks in files placed in lib/tasks ending in .rake,
# for example lib/tasks/capistrano.rake, and they will automatically be available to Rake.

require_relative "config/application"

Rails.application.load_tasks
```

`app/assets/stylesheets/application.css`:

```css
/*
 * This is a manifest file that'll be compiled into application.css.
 *
 * With Propshaft, assets are served efficiently without preprocessing steps. You can still include
 * application-wide styles in this file, but keep in mind that CSS precedence will follow the standard
 * cascading order, meaning styles declared later in the document or manifest will override earlier ones,
 * depending on specificity.
 *
 * Consider organizing styles into separate files for maintainability.
 */
```

`app/controllers/application_controller.rb`:

```rb
class ApplicationController < ActionController::Base
  # Only allow modern browsers supporting webp images, web push, badges, import maps, CSS nesting, and CSS :has.
  allow_browser versions: :modern

  # Changes to the importmap will invalidate the etag for HTML responses
  stale_when_importmap_changes
end
```

`app/helpers/application_helper.rb`:

```rb
module ApplicationHelper
end
```

`app/jobs/application_job.rb`:

```rb
class ApplicationJob < ActiveJob::Base
  # Automatically retry jobs that encountered a deadlock
  # retry_on ActiveRecord::Deadlocked

  # Most jobs are safe to ignore if the underlying records are no longer available
  # discard_on ActiveJob::DeserializationError
end
```

`app/mailers/application_mailer.rb`:

```rb
class ApplicationMailer < ActionMailer::Base
  default from: "from@example.com"
  layout "mailer"
end
```

`app/models/application_record.rb`:

```rb
class ApplicationRecord < ActiveRecord::Base
  primary_abstract_class
end
```

`app/views/layouts/application.html.erb`:

```erb
<!DOCTYPE html>
<html>
  <head>
    <title><%= content_for(:title) || "Railstest" %></title>
    <meta name="viewport" content="width=device-width,initial-scale=1">
    <meta name="apple-mobile-web-app-capable" content="yes">
    <meta name="application-name" content="Railstest">
    <meta name="mobile-web-app-capable" content="yes">
    <%= csrf_meta_tags %>
    <%= csp_meta_tag %>

    <%= yield :head %>

    <%# Enable PWA manifest for installable apps (make sure to enable in config/routes.rb too!) %>
    <%#= tag.link rel: "manifest", href: pwa_manifest_path(format: :json) %>

    <link rel="icon" href="/icon.png" type="image/png">
    <link rel="icon" href="/icon.svg" type="image/svg+xml">
    <link rel="apple-touch-icon" href="/icon.png">

    <%# Includes all stylesheet files in app/assets/stylesheets %>
    <%= stylesheet_link_tag :app, "data-turbo-track": "reload" %>
  </head>

  <body>
    <%= yield %>
  </body>
</html>
```

`app/views/layouts/mailer.html.erb`:

```erb
<!DOCTYPE html>
<html>
  <head>
    <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
    <style>
      /* Email styles need to be inline */
    </style>
  </head>

  <body>
    <%= yield %>
  </body>
</html>
```

`app/views/layouts/mailer.text.erb`:

```erb
<%= yield %>
```

`app/views/pwa/manifest.json.erb`:

```erb
{
  "name": "Railstest",
  "icons": [
    {
      "src": "/icon.png",
      "type": "image/png",
      "sizes": "512x512"
    },
    {
      "src": "/icon.png",
      "type": "image/png",
      "sizes": "512x512",
      "purpose": "maskable"
    }
  ],
  "start_url": "/",
  "display": "standalone",
  "scope": "/",
  "description": "Railstest.",
  "theme_color": "red",
  "background_color": "red"
}
```

`app/views/pwa/service-worker.js`:

```js
// Add a service worker for processing Web Push notifications:
//
// self.addEventListener("push", async (event) => {
//   const { title, options } = await event.data.json()
//   event.waitUntil(self.registration.showNotification(title, options))
// })
//
// self.addEventListener("notificationclick", function(event) {
//   event.notification.close()
//   event.waitUntil(
//     clients.matchAll({ type: "window" }).then((clientList) => {
//       for (let i = 0; i < clientList.length; i++) {
//         let client = clientList[i]
//         let clientPath = (new URL(client.url)).pathname
//
//         if (clientPath == event.notification.data.path && "focus" in client) {
//           return client.focus()
//         }
//       }
//
//       if (clients.openWindow) {
//         return clients.openWindow(event.notification.data.path)
//       }
//     })
//   )
// })
```

`bin/brakeman`:

```txt
#!/usr/bin/env ruby
require "rubygems"
require "bundler/setup"

ARGV.unshift("--ensure-latest")

load Gem.bin_path("brakeman", "brakeman")
```

`bin/bundler-audit`:

```txt
#!/usr/bin/env ruby
require_relative "../config/boot"
require "bundler/audit/cli"

ARGV.concat %w[ --config config/bundler-audit.yml ] if ARGV.empty? || ARGV.include?("check")
Bundler::Audit::CLI.start
```

`bin/ci`:

```txt
#!/usr/bin/env ruby
require_relative "../config/boot"
require "active_support/continuous_integration"

CI = ActiveSupport::ContinuousIntegration
require_relative "../config/ci.rb"
```

`bin/dev`:

```txt
#!/usr/bin/env ruby
exec "./bin/rails", "server", *ARGV
```

`bin/docker-entrypoint`:

```txt
#!/bin/bash -e

# If running the rails server then create or migrate existing database
if [ "${@: -2:1}" == "./bin/rails" ] && [ "${@: -1:1}" == "server" ]; then
  ./bin/rails db:prepare
fi

exec "${@}"
```

`bin/rails`:

```txt
#!/usr/bin/env ruby
APP_PATH = File.expand_path("../config/application", __dir__)
require_relative "../config/boot"
require "rails/commands"
```

`bin/rake`:

```txt
#!/usr/bin/env ruby
require_relative "../config/boot"
require "rake"
Rake.application.run
```

`bin/rubocop`:

```txt
#!/usr/bin/env ruby
require "rubygems"
require "bundler/setup"

# Explicit RuboCop config increases performance slightly while avoiding config confusion.
ARGV.unshift("--config", File.expand_path("../.rubocop.yml", __dir__))

load Gem.bin_path("rubocop", "rubocop")
```

`bin/setup`:

```txt
#!/usr/bin/env ruby
require "fileutils"

APP_ROOT = File.expand_path("..", __dir__)

def system!(*args)
  system(*args, exception: true)
end

FileUtils.chdir APP_ROOT do
  # This script is a way to set up or update your development environment automatically.
  # This script is idempotent, so that you can run it at any time and get an expectable outcome.
  # Add necessary setup steps to this file.

  puts "== Installing dependencies =="
  system("bundle check") || system!("bundle install")

  # puts "\n== Copying sample files =="
  # unless File.exist?("config/database.yml")
  #   FileUtils.cp "config/database.yml.sample", "config/database.yml"
  # end

  puts "\n== Preparing database =="
  system! "bin/rails db:prepare"
  system! "bin/rails db:reset" if ARGV.include?("--reset")

  puts "\n== Removing old logs and tempfiles =="
  system! "bin/rails log:clear tmp:clear"

  unless ARGV.include?("--skip-server")
    puts "\n== Starting development server =="
    STDOUT.flush # flush the output before exec(2) so that it displays
    exec "bin/dev"
  end
end
```

`bin/thrust`:

```txt
#!/usr/bin/env ruby
require "rubygems"
require "bundler/setup"

load Gem.bin_path("thruster", "thrust")
```

`config.ru`:

```ru
# This file is used by Rack-based servers to start the application.

require_relative "config/environment"

run Rails.application
Rails.application.load_server
```

`config/application.rb`:

```rb
require_relative "boot"

require "rails/all"

# Require the gems listed in Gemfile, including any gems
# you've limited to :test, :development, or :production.
Bundler.require(*Rails.groups)

module Railstest
  class Application < Rails::Application
    # Initialize configuration defaults for originally generated Rails version.
    config.load_defaults 8.1

    # Please, add to the `ignore` list any other `lib` subdirectories that do
    # not contain `.rb` files, or that should not be reloaded or eager loaded.
    # Common ones are `templates`, `generators`, or `middleware`, for example.
    config.autoload_lib(ignore: %w[assets tasks])

    # Configuration for the application, engines, and railties goes here.
    #
    # These settings can be overridden in specific environments using the files
    # in config/environments, which are processed later.
    #
    # config.time_zone = "Central Time (US & Canada)"
    # config.eager_load_paths << Rails.root.join("extras")
  end
end
```

`config/boot.rb`:

```rb
ENV["BUNDLE_GEMFILE"] ||= File.expand_path("../Gemfile", __dir__)

require "bundler/setup" # Set up gems listed in the Gemfile.
require "bootsnap/setup" # Speed up boot time by caching expensive operations.
```

`config/bundler-audit.yml`:

```yml
# Audit all gems listed in the Gemfile for known security problems by running bin/bundler-audit.
# CVEs that are not relevant to the application can be enumerated on the ignore list below.

ignore:
  - CVE-THAT-DOES-NOT-APPLY
```

`config/cable.yml`:

```yml
development:
  adapter: async

test:
  adapter: test

production:
  adapter: redis
  url: <%= ENV.fetch("REDIS_URL") { "redis://localhost:6379/1" } %>
  channel_prefix: railstest_production
```

`config/ci.rb`:

```rb
# Run using bin/ci

CI.run do
  step "Setup", "bin/setup --skip-server"

  step "Style: Ruby", "bin/rubocop"

  step "Security: Gem audit", "bin/bundler-audit"
  step "Security: Importmap vulnerability audit", "bin/importmap audit"
  step "Security: Brakeman code analysis", "bin/brakeman --quiet --no-pager --exit-on-warn --exit-on-error"
  step "Tests: Rails", "bin/rails test"
  step "Tests: Seeds", "env RAILS_ENV=test bin/rails db:seed:replant"

  # Optional: Run system tests
  # step "Tests: System", "bin/rails test:system"

  # Optional: set a green GitHub commit status to unblock PR merge.
  # Requires the `gh` CLI and `gh extension install basecamp/gh-signoff`.
  # if success?
  #   step "Signoff: All systems go. Ready for merge and deploy.", "gh signoff"
  # else
  #   failure "Signoff: CI failed. Do not merge or deploy.", "Fix the issues and try again."
  # end
end
```

`config/credentials.yml.enc`:

```enc
5bDQYHnkNBdYBkwJ4z0Nxbw2XYcgZ8XsFmGuuisBRrDmvWX+B++YKBr+lfZBaPChQj9EPCKI4VFFGUli3tBxMESS7/5FApMbezjQ57U1Na7dHkLVpvixhf0bv/dywtsoTTeiL6OJh/3R1cuvD6ntfX+oe/Q6WdYP/YFpupvDQPPw/qfJ+9cHa3OJ9fIj9W4Obpu8i41afNpFtYi9BZuKiqhCnWLz1L3ys2dzi2bXmBN2pV6+mXc6OMVJ0fIHaO+UJravM90uATeGjBnMvfVfB3+3jW74XT9CLIomAi5vTsk6JbNnivKNnEmVlCdgWy3HZeC3sniU5bXNLV0DYGgNY7b1T6Ifhx9GHm6n7oJI1rtTxlFKyyGSZRSvqjx0z9lTU8h8sm87McKkk+F/fsXymT/aYdghD78CMCxMWCRIwCuI3UPQxKA1ZIqOG9Z3hz61DRmj+FOknESEEuOZuCZReovokSw67HhbRCVV+ZouFzQZ6yGIDe7OjHAC--1dXA2NGXbG02UgAV--cjjK19H/rjbFM/HeOJIUOw==
```

`config/database.yml`:

```yml
# SQLite. Versions 3.8.0 and up are supported.
#   gem install sqlite3
#
#   Ensure the SQLite 3 gem is defined in your Gemfile
#   gem "sqlite3"
#
default: &default
  adapter: sqlite3
  max_connections: <%= ENV.fetch("RAILS_MAX_THREADS") { 5 } %>
  timeout: 5000

development:
  <<: *default
  database: storage/development.sqlite3

# Warning: The database defined as "test" will be erased and
# re-generated from your development database when you run "rake".
# Do not set this db to the same as development or production.
test:
  <<: *default
  database: storage/test.sqlite3

# Store production database in the storage/ directory, which by default
# is mounted as a persistent Docker volume in config/deploy.yml.
production:
  primary:
    <<: *default
    database: storage/production.sqlite3
  cache:
    <<: *default
    database: storage/production_cache.sqlite3
    migrations_paths: db/cache_migrate
  queue:
    <<: *default
    database: storage/production_queue.sqlite3
    migrations_paths: db/queue_migrate
  cable:
    <<: *default
    database: storage/production_cable.sqlite3
    migrations_paths: db/cable_migrate
```

`config/deploy.yml`:

```yml
# Name of your application. Used to uniquely configure containers.
service: railstest

# Name of the container image (use your-user/app-name on external registries).
image: railstest

# Deploy to these servers.
servers:
  web:
    - 192.168.0.1
  # job:
  #   hosts:
  #     - 192.168.0.1
  #   cmd: bin/jobs

# Enable SSL auto certification via Let's Encrypt and allow for multiple apps on a single web server.
# If used with Cloudflare, set encryption mode in SSL/TLS setting to "Full" to enable CF-to-app encryption.
#
# Using an SSL proxy like this requires turning on config.assume_ssl and config.force_ssl in production.rb!
#
# Don't use this when deploying to multiple web servers (then you have to terminate SSL at your load balancer).
#
# proxy:
#   ssl: true
#   host: app.example.com

# Where you keep your container images.
registry:
  # Alternatives: hub.docker.com / registry.digitalocean.com / ghcr.io / ...
  server: localhost:5555

  # Needed for authenticated registries.
  # username: your-user

  # Always use an access token rather than real password when possible.
  # password:
  #   - KAMAL_REGISTRY_PASSWORD

# Inject ENV variables into containers (secrets come from .kamal/secrets).
env:
  secret:
    - RAILS_MASTER_KEY
  clear:
    # Run the Solid Queue Supervisor inside the web server's Puma process to do jobs.
    # When you start using multiple servers, you should split out job processing to a dedicated machine.
    SOLID_QUEUE_IN_PUMA: true

    # Set number of processes dedicated to Solid Queue (default: 1)
    # JOB_CONCURRENCY: 3

    # Set number of cores available to the application on each server (default: 1).
    # WEB_CONCURRENCY: 2

    # Match this to any external database server to configure Active Record correctly
    # Use railstest-db for a db accessory server on same machine via local kamal docker network.
    # DB_HOST: 192.168.0.2

    # Log everything from Rails
    # RAILS_LOG_LEVEL: debug

# Aliases are triggered with "bin/kamal <alias>". You can overwrite arguments on invocation:
# "bin/kamal logs -r job" will tail logs from the first server in the job section.
aliases:
  console: app exec --interactive --reuse "bin/rails console"
  shell: app exec --interactive --reuse "bash"
  logs: app logs -f
  dbc: app exec --interactive --reuse "bin/rails dbconsole --include-password"

# Use a persistent storage volume for sqlite database files and local Active Storage files.
# Recommended to change this to a mounted volume path that is backed up off server.
volumes:
  - "railstest_storage:/rails/storage"

# Bridge fingerprinted assets, like JS and CSS, between versions to avoid
# hitting 404 on in-flight requests. Combines all files from new and old
# version inside the asset_path.
asset_path: /rails/public/assets

# Configure the image builder.
builder:
  arch: amd64

  # # Build image via remote server (useful for faster amd64 builds on arm64 computers)
  # remote: ssh://docker@docker-builder-server
  #
  # # Pass arguments and secrets to the Docker build process
  # args:
  #   RUBY_VERSION: ruby-3.4.8
  # secrets:
  #   - GITHUB_TOKEN
  #   - RAILS_MASTER_KEY

# Use a different ssh user than root
# ssh:
#   user: app

# Use accessory services (secrets come from .kamal/secrets).
# accessories:
#   db:
#     image: mysql:8.0
#     host: 192.168.0.2
#     # Change to 3306 to expose port to the world instead of just local network.
#     port: "127.0.0.1:3306:3306"
#     env:
#       clear:
#         MYSQL_ROOT_HOST: '%'
#       secret:
#         - MYSQL_ROOT_PASSWORD
#     files:
#       - config/mysql/production.cnf:/etc/mysql/my.cnf
#       - db/production.sql:/docker-entrypoint-initdb.d/setup.sql
#     directories:
#       - data:/var/lib/mysql
#   redis:
#     image: valkey/valkey:8
#     host: 192.168.0.2
#     port: 6379
#     directories:
#       - data:/data
```

`config/environment.rb`:

```rb
# Load the Rails application.
require_relative "application"

# Initialize the Rails application.
Rails.application.initialize!
```

`config/environments/development.rb`:

```rb
require "active_support/core_ext/integer/time"

Rails.application.configure do
  # Settings specified here will take precedence over those in config/application.rb.

  # Make code changes take effect immediately without server restart.
  config.enable_reloading = true

  # Do not eager load code on boot.
  config.eager_load = false

  # Show full error reports.
  config.consider_all_requests_local = true

  # Enable server timing.
  config.server_timing = true

  # Enable/disable Action Controller caching. By default Action Controller caching is disabled.
  # Run rails dev:cache to toggle Action Controller caching.
  if Rails.root.join("tmp/caching-dev.txt").exist?
    config.action_controller.perform_caching = true
    config.action_controller.enable_fragment_cache_logging = true
    config.public_file_server.headers = { "cache-control" => "public, max-age=#{2.days.to_i}" }
  else
    config.action_controller.perform_caching = false
  end

  # Change to :null_store to avoid any caching.
  config.cache_store = :memory_store

  # Store uploaded files on the local file system (see config/storage.yml for options).
  config.active_storage.service = :local

  # Don't care if the mailer can't send.
  config.action_mailer.raise_delivery_errors = false

  # Make template changes take effect immediately.
  config.action_mailer.perform_caching = false

  # Set localhost to be used by links generated in mailer templates.
  config.action_mailer.default_url_options = { host: "localhost", port: 3000 }

  # Print deprecation notices to the Rails logger.
  config.active_support.deprecation = :log

  # Raise an error on page load if there are pending migrations.
  config.active_record.migration_error = :page_load

  # Highlight code that triggered database queries in logs.
  config.active_record.verbose_query_logs = true

  # Append comments with runtime information tags to SQL queries in logs.
  config.active_record.query_log_tags_enabled = true

  # Highlight code that enqueued background job in logs.
  config.active_job.verbose_enqueue_logs = true

  # Highlight code that triggered redirect in logs.
  config.action_dispatch.verbose_redirect_logs = true

  # Suppress logger output for asset requests.
  config.assets.quiet = true

  # Raises error for missing translations.
  # config.i18n.raise_on_missing_translations = true

  # Annotate rendered view with file names.
  config.action_view.annotate_rendered_view_with_filenames = true

  # Uncomment if you wish to allow Action Cable access from any origin.
  # config.action_cable.disable_request_forgery_protection = true

  # Raise error when a before_action's only/except options reference missing actions.
  config.action_controller.raise_on_missing_callback_actions = true

  # Apply autocorrection by RuboCop to files generated by `bin/rails generate`.
  # config.generators.apply_rubocop_autocorrect_after_generate!
end
```

`config/environments/production.rb`:

```rb
require "active_support/core_ext/integer/time"

Rails.application.configure do
  # Settings specified here will take precedence over those in config/application.rb.

  # Code is not reloaded between requests.
  config.enable_reloading = false

  # Eager load code on boot for better performance and memory savings (ignored by Rake tasks).
  config.eager_load = true

  # Full error reports are disabled.
  config.consider_all_requests_local = false

  # Turn on fragment caching in view templates.
  config.action_controller.perform_caching = true

  # Cache assets for far-future expiry since they are all digest stamped.
  config.public_file_server.headers = { "cache-control" => "public, max-age=#{1.year.to_i}" }

  # Enable serving of images, stylesheets, and JavaScripts from an asset server.
  # config.asset_host = "http://assets.example.com"

  # Store uploaded files on the local file system (see config/storage.yml for options).
  config.active_storage.service = :local

  # Assume all access to the app is happening through a SSL-terminating reverse proxy.
  # config.assume_ssl = true

  # Force all access to the app over SSL, use Strict-Transport-Security, and use secure cookies.
  # config.force_ssl = true

  # Skip http-to-https redirect for the default health check endpoint.
  # config.ssl_options = { redirect: { exclude: ->(request) { request.path == "/up" } } }

  # Log to STDOUT with the current request id as a default log tag.
  config.log_tags = [ :request_id ]
  config.logger   = ActiveSupport::TaggedLogging.logger(STDOUT)

  # Change to "debug" to log everything (including potentially personally-identifiable information!).
  config.log_level = ENV.fetch("RAILS_LOG_LEVEL", "info")

  # Prevent health checks from clogging up the logs.
  config.silence_healthcheck_path = "/up"

  # Don't log any deprecations.
  config.active_support.report_deprecations = false

  # Replace the default in-process memory cache store with a durable alternative.
  # config.cache_store = :mem_cache_store

  # Replace the default in-process and non-durable queuing backend for Active Job.
  # config.active_job.queue_adapter = :resque

  # Ignore bad email addresses and do not raise email delivery errors.
  # Set this to true and configure the email server for immediate delivery to raise delivery errors.
  # config.action_mailer.raise_delivery_errors = false

  # Set host to be used by links generated in mailer templates.
  config.action_mailer.default_url_options = { host: "example.com" }

  # Specify outgoing SMTP server. Remember to add smtp/* credentials via bin/rails credentials:edit.
  # config.action_mailer.smtp_settings = {
  #   user_name: Rails.application.credentials.dig(:smtp, :user_name),
  #   password: Rails.application.credentials.dig(:smtp, :password),
  #   address: "smtp.example.com",
  #   port: 587,
  #   authentication: :plain
  # }

  # Enable locale fallbacks for I18n (makes lookups for any locale fall back to
  # the I18n.default_locale when a translation cannot be found).
  config.i18n.fallbacks = true

  # Do not dump schema after migrations.
  config.active_record.dump_schema_after_migration = false

  # Only use :id for inspections in production.
  config.active_record.attributes_for_inspect = [ :id ]

  # Enable DNS rebinding protection and other `Host` header attacks.
  # config.hosts = [
  #   "example.com",     # Allow requests from example.com
  #   /.*\.example\.com/ # Allow requests from subdomains like `www.example.com`
  # ]
  #
  # Skip DNS rebinding protection for the default health check endpoint.
  # config.host_authorization = { exclude: ->(request) { request.path == "/up" } }
end
```

`config/environments/test.rb`:

```rb
# The test environment is used exclusively to run your application's
# test suite. You never need to work with it otherwise. Remember that
# your test database is "scratch space" for the test suite and is wiped
# and recreated between test runs. Don't rely on the data there!

Rails.application.configure do
  # Settings specified here will take precedence over those in config/application.rb.

  # While tests run files are not watched, reloading is not necessary.
  config.enable_reloading = false

  # Eager loading loads your entire application. When running a single test locally,
  # this is usually not necessary, and can slow down your test suite. However, it's
  # recommended that you enable it in continuous integration systems to ensure eager
  # loading is working properly before deploying your code.
  config.eager_load = ENV["CI"].present?

  # Configure public file server for tests with cache-control for performance.
  config.public_file_server.headers = { "cache-control" => "public, max-age=3600" }

  # Show full error reports.
  config.consider_all_requests_local = true
  config.cache_store = :null_store

  # Render exception templates for rescuable exceptions and raise for other exceptions.
  config.action_dispatch.show_exceptions = :rescuable

  # Disable request forgery protection in test environment.
  config.action_controller.allow_forgery_protection = false

  # Store uploaded files on the local file system in a temporary directory.
  config.active_storage.service = :test

  # Tell Action Mailer not to deliver emails to the real world.
  # The :test delivery method accumulates sent emails in the
  # ActionMailer::Base.deliveries array.
  config.action_mailer.delivery_method = :test

  # Set host to be used by links generated in mailer templates.
  config.action_mailer.default_url_options = { host: "example.com" }

  # Print deprecation notices to the stderr.
  config.active_support.deprecation = :stderr

  # Raises error for missing translations.
  # config.i18n.raise_on_missing_translations = true

  # Annotate rendered view with file names.
  # config.action_view.annotate_rendered_view_with_filenames = true

  # Raise error when a before_action's only/except options reference missing actions.
  config.action_controller.raise_on_missing_callback_actions = true
end
```

`config/initializers/assets.rb`:

```rb
# Be sure to restart your server when you modify this file.

# Version of your assets, change this if you want to expire all your assets.
Rails.application.config.assets.version = "1.0"

# Add additional assets to the asset load path.
# Rails.application.config.assets.paths << Emoji.images_path
```

`config/initializers/content_security_policy.rb`:

```rb
# Be sure to restart your server when you modify this file.

# Define an application-wide content security policy.
# See the Securing Rails Applications Guide for more information:
# https://guides.rubyonrails.org/security.html#content-security-policy-header

# Rails.application.configure do
#   config.content_security_policy do |policy|
#     policy.default_src :self, :https
#     policy.font_src    :self, :https, :data
#     policy.img_src     :self, :https, :data
#     policy.object_src  :none
#     policy.script_src  :self, :https
#     policy.style_src   :self, :https
#     # Specify URI for violation reports
#     # policy.report_uri "/csp-violation-report-endpoint"
#   end
#
#   # Generate session nonces for permitted importmap, inline scripts, and inline styles.
#   config.content_security_policy_nonce_generator = ->(request) { request.session.id.to_s }
#   config.content_security_policy_nonce_directives = %w(script-src style-src)
#
#   # Automatically add `nonce` to `javascript_tag`, `javascript_include_tag`, and `stylesheet_link_tag`
#   # if the corresponding directives are specified in `content_security_policy_nonce_directives`.
#   # config.content_security_policy_nonce_auto = true
#
#   # Report violations without enforcing the policy.
#   # config.content_security_policy_report_only = true
# end
```

`config/initializers/filter_parameter_logging.rb`:

```rb
# Be sure to restart your server when you modify this file.

# Configure parameters to be partially matched (e.g. passw matches password) and filtered from the log file.
# Use this to limit dissemination of sensitive information.
# See the ActiveSupport::ParameterFilter documentation for supported notations and behaviors.
Rails.application.config.filter_parameters += [
  :passw, :email, :secret, :token, :_key, :crypt, :salt, :certificate, :otp, :ssn, :cvv, :cvc
]
```

`config/initializers/inflections.rb`:

```rb
# Be sure to restart your server when you modify this file.

# Add new inflection rules using the following format. Inflections
# are locale specific, and you may define rules for as many different
# locales as you wish. All of these examples are active by default:
# ActiveSupport::Inflector.inflections(:en) do |inflect|
#   inflect.plural /^(ox)$/i, "\\1en"
#   inflect.singular /^(ox)en/i, "\\1"
#   inflect.irregular "person", "people"
#   inflect.uncountable %w( fish sheep )
# end

# These inflection rules are supported but not enabled by default:
# ActiveSupport::Inflector.inflections(:en) do |inflect|
#   inflect.acronym "RESTful"
# end
```

`config/locales/en.yml`:

```yml
# Files in the config/locales directory are used for internationalization and
# are automatically loaded by Rails. If you want to use locales other than
# English, add the necessary files in this directory.
#
# To use the locales, use `I18n.t`:
#
#     I18n.t "hello"
#
# In views, this is aliased to just `t`:
#
#     <%= t("hello") %>
#
# To use a different locale, set it with `I18n.locale`:
#
#     I18n.locale = :es
#
# This would use the information in config/locales/es.yml.
#
# To learn more about the API, please read the Rails Internationalization guide
# at https://guides.rubyonrails.org/i18n.html.
#
# Be aware that YAML interprets the following case-insensitive strings as
# booleans: `true`, `false`, `on`, `off`, `yes`, `no`. Therefore, these strings
# must be quoted to be interpreted as strings. For example:
#
#     en:
#       "yes": yup
#       enabled: "ON"

en:
  hello: "Hello world"
```

`config/puma.rb`:

```rb
# This configuration file will be evaluated by Puma. The top-level methods that
# are invoked here are part of Puma's configuration DSL. For more information
# about methods provided by the DSL, see https://puma.io/puma/Puma/DSL.html.
#
# Puma starts a configurable number of processes (workers) and each process
# serves each request in a thread from an internal thread pool.
#
# You can control the number of workers using ENV["WEB_CONCURRENCY"]. You
# should only set this value when you want to run 2 or more workers. The
# default is already 1. You can set it to `auto` to automatically start a worker
# for each available processor.
#
# The ideal number of threads per worker depends both on how much time the
# application spends waiting for IO operations and on how much you wish to
# prioritize throughput over latency.
#
# As a rule of thumb, increasing the number of threads will increase how much
# traffic a given process can handle (throughput), but due to CRuby's
# Global VM Lock (GVL) it has diminishing returns and will degrade the
# response time (latency) of the application.
#
# The default is set to 3 threads as it's deemed a decent compromise between
# throughput and latency for the average Rails application.
#
# Any libraries that use a connection pool or another resource pool should
# be configured to provide at least as many connections as the number of
# threads. This includes Active Record's `pool` parameter in `database.yml`.
threads_count = ENV.fetch("RAILS_MAX_THREADS", 3)
threads threads_count, threads_count

# Specifies the `port` that Puma will listen on to receive requests; default is 3000.
port ENV.fetch("PORT", 3000)

# Allow puma to be restarted by `bin/rails restart` command.
plugin :tmp_restart

# Run the Solid Queue supervisor inside of Puma for single-server deployments.
plugin :solid_queue if ENV["SOLID_QUEUE_IN_PUMA"]

# Specify the PID file. Defaults to tmp/pids/server.pid in development.
# In other environments, only set the PID file if requested.
pidfile ENV["PIDFILE"] if ENV["PIDFILE"]
```

`config/routes.rb`:

```rb
Rails.application.routes.draw do
  # Define your application routes per the DSL in https://guides.rubyonrails.org/routing.html

  # Reveal health status on /up that returns 200 if the app boots with no exceptions, otherwise 500.
  # Can be used by load balancers and uptime monitors to verify that the app is live.
  get "up" => "rails/health#show", as: :rails_health_check

  # Render dynamic PWA files from app/views/pwa/* (remember to link manifest in application.html.erb)
  # get "manifest" => "rails/pwa#manifest", as: :pwa_manifest
  # get "service-worker" => "rails/pwa#service_worker", as: :pwa_service_worker

  # Defines the root path route ("/")
  # root "posts#index"
end
```

`config/storage.yml`:

```yml
test:
  service: Disk
  root: <%= Rails.root.join("tmp/storage") %>

local:
  service: Disk
  root: <%= Rails.root.join("storage") %>

# Use bin/rails credentials:edit to set the AWS secrets (as aws:access_key_id|secret_access_key)
# amazon:
#   service: S3
#   access_key_id: <%= Rails.application.credentials.dig(:aws, :access_key_id) %>
#   secret_access_key: <%= Rails.application.credentials.dig(:aws, :secret_access_key) %>
#   region: us-east-1
#   bucket: your_own_bucket-<%= Rails.env %>

# Remember not to checkin your GCS keyfile to a repository
# google:
#   service: GCS
#   project: your_project
#   credentials: <%= Rails.root.join("path/to/gcs.keyfile") %>
#   bucket: your_own_bucket-<%= Rails.env %>

# mirror:
#   service: Mirror
#   primary: local
#   mirrors: [ amazon, google, microsoft ]
```

`db/seeds.rb`:

```rb
# This file should ensure the existence of records required to run the application in every environment (production,
# development, test). The code here should be idempotent so that it can be executed at any point in every environment.
# The data can then be loaded with the bin/rails db:seed command (or created alongside the database with db:setup).
#
# Example:
#
#   ["Action", "Comedy", "Drama", "Horror"].each do |genre_name|
#     MovieGenre.find_or_create_by!(name: genre_name)
#   end
```

`log/.keep`:

```txt

```

`public/400.html`:

```html
<!doctype html>

<html lang="en">

  <head>

    <title>The server cannot process the request due to a client error (400 Bad Request)</title>

    <meta charset="utf-8">
    <meta name="viewport" content="initial-scale=1, width=device-width">
    <meta name="robots" content="noindex, nofollow">

    <style>

      *, *::before, *::after {
        box-sizing: border-box;
      }

      * {
        margin: 0;
      }

      html {
        font-size: 16px;
      }

      body {
        background: #FFF;
        color: #261B23;
        display: grid;
        font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, Aptos, Roboto, "Segoe UI", "Helvetica Neue", Helvetica, Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji";
        font-size: clamp(1rem, 2.5vw, 2rem);
        -webkit-font-smoothing: antialiased;
        font-style: normal;
        font-weight: 400;
        letter-spacing: -0.0025em;
        line-height: 1.4;
        min-height: 100dvh;
        place-items: center;
        text-rendering: optimizeLegibility;
        -webkit-text-size-adjust: 100%;
      }

      #error-description {
        fill: #d30001;
      }

      #error-id {
        fill: #f0eff0;
      }

      @media (prefers-color-scheme: dark) {
        body {
          background: #101010;
          color: #e0e0e0;
        }

        #error-description {
          fill: #FF6161;
        }

        #error-id {
          fill: #2c2c2c;
        }
      }

      a {
        color: inherit;
        font-weight: 700;
        text-decoration: underline;
        text-underline-offset: 0.0925em;
      }

      b, strong {
        font-weight: 700;
      }

      i, em {
        font-style: italic;
      }

      main {
        display: grid;
        gap: 1em;
        padding: 2em;
        place-items: center;
        text-align: center;
      }

      main header {
        width: min(100%, 12em);
      }

      main header svg {
        height: auto;
        max-width: 100%;
        width: 100%;
      }

      main article {
        width: min(100%, 30em);
      }

      main article p {
        font-size: 75%;
      }

      main article br {
        display: none;

        @media(min-width: 48em) {
          display: inline;
        }
      }

    </style>

  </head>

  <body>

    <!-- This file lives in public/400.html -->

    <main>
      <header>
        <svg height="172" viewBox="0 0 480 172" width="480" xmlns="http://www.w3.org/2000/svg"><path d="m124.48 3.00509-45.6889 100.02991h26.2239v-28.1168h38.119v28.1168h21.628v35.145h-21.628v30.82h-37.308v-30.82h-72.1833v-31.901l50.2851-103.27391zm115.583 168.69891c-40.822 0-64.884-35.146-64.884-85.7015 0-50.5554 24.062-85.700907 64.884-85.700907 40.823 0 64.884 35.145507 64.884 85.700907 0 50.5555-24.061 85.7015-64.884 85.7015zm0-133.2831c-17.572 0-22.709 21.8984-22.709 47.5816 0 25.6835 5.137 47.5815 22.709 47.5815 17.303 0 22.71-21.898 22.71-47.5815 0-25.6832-5.407-47.5816-22.71-47.5816zm140.456 133.2831c-40.823 0-64.884-35.146-64.884-85.7015 0-50.5554 24.061-85.700907 64.884-85.700907 40.822 0 64.884 35.145507 64.884 85.700907 0 50.5555-24.062 85.7015-64.884 85.7015zm0-133.2831c-17.573 0-22.71 21.8984-22.71 47.5816 0 25.6835 5.137 47.5815 22.71 47.5815 17.302 0 22.709-21.898 22.709-47.5815 0-25.6832-5.407-47.5816-22.709-47.5816z" id="error-id"/><path d="m123.606 85.4445c3.212 1.0523 5.538 4.2089 5.538 8.0301 0 6.1472-4.209 9.5254-11.298 9.5254h-15.617v-34.0033h14.565c7.089 0 11.353 3.1566 11.353 9.2484 0 3.6551-2.049 6.3134-4.541 7.1994zm-12.904-2.9905h5.095c2.603 0 3.988-.9968 3.988-3.1013 0-2.1044-1.385-3.0459-3.988-3.0459h-5.095zm0 6.6456v6.5902h5.981c2.492 0 3.877-1.3291 3.877-3.2674 0-2.049-1.385-3.3228-3.877-3.3228zm43.786 13.9004h-8.362v-1.274c-.831.831-3.323 1.717-5.981 1.717-4.929 0-9.083-2.769-9.083-8.0301 0-4.818 4.154-7.9193 9.581-7.9193 2.049 0 4.486.6646 5.483 1.3845v-1.606c0-1.606-.942-2.9905-3.046-2.9905-1.606 0-2.548.7199-2.935 1.8275h-8.197c.72-4.8181 4.985-8.6393 11.409-8.6393 7.088 0 11.131 3.7659 11.131 10.2453zm-8.362-6.9779v-1.4399c-.554-1.0522-2.049-1.7167-3.655-1.7167-1.717 0-3.434.7199-3.434 2.3813 0 1.7168 1.717 2.4367 3.434 2.4367 1.606 0 3.101-.6645 3.655-1.6614zm27.996 6.9779v-1.994c-1.163 1.329-3.599 2.548-6.147 2.548-7.199 0-11.131-5.8151-11.131-13.0145s3.932-13.0143 11.131-13.0143c2.548 0 4.984 1.2184 6.147 2.5475v-13.0697h8.695v35.997zm0-9.1931v-6.5902c-.664-1.3291-2.159-2.326-3.821-2.326-2.99 0-4.763 2.4368-4.763 5.6488s1.773 5.5934 4.763 5.5934c1.717 0 3.157-.9415 3.821-2.326zm35.471-2.049h-3.101v11.2421h-8.806v-34.0033h15.285c7.31 0 12.35 4.1535 12.35 11.5744 0 5.1503-2.603 8.6947-6.757 10.2453l7.975 12.1836h-9.858zm-3.101-15.2849v8.1962h5.538c3.156 0 4.596-1.606 4.596-4.0981s-1.44-4.0981-4.596-4.0981zm36.957 17.8323h8.03c-.886 5.7597-5.206 9.2487-11.685 9.2487-7.643 0-12.682-5.2613-12.682-13.0145 0-7.6978 5.316-13.0143 12.515-13.0143 7.643 0 11.962 5.095 11.962 12.5159v2.1598h-16.115c.277 2.9905 1.827 4.5965 4.32 4.5965 1.772 0 3.156-.7753 3.655-2.4921zm-3.822-10.0237c-2.049 0-3.433 1.2737-3.987 3.5997h7.532c-.111-2.0491-1.385-3.5997-3.545-3.5997zm30.98 27.5234v-10.799c-1.163 1.329-3.6 2.548-6.147 2.548-7.2 0-11.132-5.9259-11.132-13.0145 0-7.144 3.932-13.0143 11.132-13.0143 2.547 0 4.984 1.2184 6.147 2.5475v-1.9937h8.695v33.726zm0-17.9981v-6.5902c-.665-1.3291-2.105-2.326-3.821-2.326-2.991 0-4.763 2.4368-4.763 5.6488s1.772 5.5934 4.763 5.5934c1.661 0 3.156-.9415 3.821-2.326zm36.789-15.7279v24.921h-8.695v-2.16c-1.329 1.551-3.821 2.714-6.646 2.714-5.482 0-8.75-3.5999-8.75-9.1379v-16.3371h8.64v14.288c0 2.1045.996 3.5997 3.212 3.5997 1.606 0 3.101-1.0522 3.544-2.769v-15.1187zm19.084 16.2263h8.03c-.886 5.7597-5.206 9.2487-11.685 9.2487-7.643 0-12.682-5.2613-12.682-13.0145 0-7.6978 5.316-13.0143 12.515-13.0143 7.643 0 11.963 5.095 11.963 12.5159v2.1598h-16.116c.277 2.9905 1.828 4.5965 4.32 4.5965 1.772 0 3.156-.7753 3.655-2.4921zm-3.822-10.0237c-2.049 0-3.433 1.2737-3.987 3.5997h7.532c-.111-2.0491-1.385-3.5997-3.545-3.5997zm13.428 11.0206h8.474c.387 1.3845 1.606 2.1598 3.156 2.1598 1.44 0 2.548-.5538 2.548-1.7168 0-.9414-.72-1.2737-1.939-1.5506l-4.873-.9969c-4.154-.886-6.867-2.8797-6.867-7.2547 0-5.3165 4.762-8.4178 10.633-8.4178 6.812 0 10.522 3.1567 11.297 8.0855h-8.03c-.277-1.0522-1.052-1.9937-3.046-1.9937-1.273 0-2.326.5538-2.326 1.6614 0 .7753.554 1.163 1.717 1.3845l4.929 1.163c4.541 1.0522 6.978 3.4335 6.978 7.4763 0 5.3168-4.818 8.2518-10.91 8.2518-6.369 0-10.965-2.88-11.741-8.2518zm27.538-.8861v-9.5807h-3.655v-6.7564h3.655v-6.8671h8.584v6.8671h5.205v6.7564h-5.205v8.307c0 1.9383.941 2.769 2.658 2.769.941 0 1.993-.2216 2.769-.5538v7.3654c-.997.443-2.88.775-4.818.775-5.871 0-9.193-2.769-9.193-9.0819z" id="error-description"/></svg>
      </header>
      <article>
        <p><strong>The server cannot process the request due to a client error.</strong> Please check the request and try again. If you're the application owner check the logs for more information.</p>
      </article>
    </main>

  </body>

</html>
```

`public/404.html`:

```html
<!doctype html>

<html lang="en">

  <head>

    <title>The page you were looking for doesn't exist (404 Not found)</title>

    <meta charset="utf-8">
    <meta name="viewport" content="initial-scale=1, width=device-width">
    <meta name="robots" content="noindex, nofollow">

    <style>

      *, *::before, *::after {
        box-sizing: border-box;
      }

      * {
        margin: 0;
      }

      html {
        font-size: 16px;
      }

      body {
        background: #FFF;
        color: #261B23;
        display: grid;
        font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, Aptos, Roboto, "Segoe UI", "Helvetica Neue", Helvetica, Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji";
        font-size: clamp(1rem, 2.5vw, 2rem);
        -webkit-font-smoothing: antialiased;
        font-style: normal;
        font-weight: 400;
        letter-spacing: -0.0025em;
        line-height: 1.4;
        min-height: 100dvh;
        place-items: center;
        text-rendering: optimizeLegibility;
        -webkit-text-size-adjust: 100%;
      }

      #error-description {
        fill: #d30001;
      }

      #error-id {
        fill: #f0eff0;
      }

      @media (prefers-color-scheme: dark) {
        body {
          background: #101010;
          color: #e0e0e0;
        }

        #error-description {
          fill: #FF6161;
        }

        #error-id {
          fill: #2c2c2c;
        }
      }

      a {
        color: inherit;
        font-weight: 700;
        text-decoration: underline;
        text-underline-offset: 0.0925em;
      }

      b, strong {
        font-weight: 700;
      }

      i, em {
        font-style: italic;
      }

      main {
        display: grid;
        gap: 1em;
        padding: 2em;
        place-items: center;
        text-align: center;
      }

      main header {
        width: min(100%, 12em);
      }

      main header svg {
        height: auto;
        max-width: 100%;
        width: 100%;
      }

      main article {
        width: min(100%, 30em);
      }

      main article p {
        font-size: 75%;
      }

      main article br {
        display: none;

        @media(min-width: 48em) {
          display: inline;
        }
      }

    </style>

  </head>

  <body>

    <!-- This file lives in public/404.html -->

    <main>
      <header>
        <svg height="172" viewBox="0 0 480 172" width="480" xmlns="http://www.w3.org/2000/svg"><path d="m124.48 3.00509-45.6889 100.02991h26.2239v-28.1168h38.119v28.1168h21.628v35.145h-21.628v30.82h-37.308v-30.82h-72.1833v-31.901l50.2851-103.27391zm115.583 168.69891c-40.822 0-64.884-35.146-64.884-85.7015 0-50.5554 24.062-85.700907 64.884-85.700907 40.823 0 64.884 35.145507 64.884 85.700907 0 50.5555-24.061 85.7015-64.884 85.7015zm0-133.2831c-17.572 0-22.709 21.8984-22.709 47.5816 0 25.6835 5.137 47.5815 22.709 47.5815 17.303 0 22.71-21.898 22.71-47.5815 0-25.6832-5.407-47.5816-22.71-47.5816zm165.328-35.41581-45.689 100.02991h26.224v-28.1168h38.119v28.1168h21.628v35.145h-21.628v30.82h-37.308v-30.82h-72.184v-31.901l50.285-103.27391z" id="error-id"/><path d="m157.758 68.9967v34.0033h-7.199l-14.233-19.8814v19.8814h-8.584v-34.0033h8.307l13.125 18.7184v-18.7184zm28.454 21.5428c0 7.6978-5.15 13.0145-12.737 13.0145-7.532 0-12.738-5.3167-12.738-13.0145s5.206-13.0143 12.738-13.0143c7.587 0 12.737 5.3165 12.737 13.0143zm-8.528 0c0-3.4336-1.496-5.8703-4.209-5.8703-2.659 0-4.154 2.4367-4.154 5.8703s1.495 5.8149 4.154 5.8149c2.713 0 4.209-2.3813 4.209-5.8149zm13.184 3.8766v-9.5807h-3.655v-6.7564h3.655v-6.8671h8.584v6.8671h5.205v6.7564h-5.205v8.307c0 1.9383.941 2.769 2.658 2.769.941 0 1.994-.2216 2.769-.5538v7.3654c-.997.443-2.88.775-4.818.775-5.87 0-9.193-2.769-9.193-9.0819zm37.027 8.5839h-8.806v-34.0033h23.924v7.6978h-15.118v6.7564h13.9v7.5316h-13.9zm41.876-12.4605c0 7.6978-5.15 13.0145-12.737 13.0145-7.532 0-12.738-5.3167-12.738-13.0145s5.206-13.0143 12.738-13.0143c7.587 0 12.737 5.3165 12.737 13.0143zm-8.529 0c0-3.4336-1.495-5.8703-4.208-5.8703-2.659 0-4.154 2.4367-4.154 5.8703s1.495 5.8149 4.154 5.8149c2.713 0 4.208-2.3813 4.208-5.8149zm35.337-12.4605v24.921h-8.695v-2.16c-1.329 1.551-3.821 2.714-6.646 2.714-5.482 0-8.75-3.5999-8.75-9.1379v-16.3371h8.64v14.288c0 2.1045.997 3.5997 3.212 3.5997 1.606 0 3.101-1.0522 3.544-2.769v-15.1187zm4.076 24.921v-24.921h8.694v2.1598c1.385-1.5506 3.822-2.7136 6.701-2.7136 5.538 0 8.806 3.5997 8.806 9.1377v16.3371h-8.639v-14.2327c0-2.049-1.053-3.5443-3.268-3.5443-1.717 0-3.156.9969-3.6 2.7136v15.0634zm44.113 0v-1.994c-1.163 1.329-3.6 2.548-6.147 2.548-7.2 0-11.132-5.8151-11.132-13.0145s3.932-13.0143 11.132-13.0143c2.547 0 4.984 1.2184 6.147 2.5475v-13.0697h8.695v35.997zm0-9.1931v-6.5902c-.665-1.3291-2.16-2.326-3.821-2.326-2.991 0-4.763 2.4368-4.763 5.6488s1.772 5.5934 4.763 5.5934c1.717 0 3.156-.9415 3.821-2.326z" id="error-description"/></svg>
      </header>
      <article>
        <p><strong>The page you were looking for doesn't exist.</strong> You may have mistyped the address or the page may have moved. If you're the application owner check the logs for more information.</p>
      </article>
    </main>

  </body>

</html>
```

`public/406-unsupported-browser.html`:

```html
<!doctype html>

<html lang="en">

  <head>

    <title>Your browser is not supported (406 Not Acceptable)</title>

    <meta charset="utf-8">
    <meta name="viewport" content="initial-scale=1, width=device-width">
    <meta name="robots" content="noindex, nofollow">

    <style>

      *, *::before, *::after {
        box-sizing: border-box;
      }

      * {
        margin: 0;
      }

      html {
        font-size: 16px;
      }

      body {
        background: #FFF;
        color: #261B23;
        display: grid;
        font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, Aptos, Roboto, "Segoe UI", "Helvetica Neue", Helvetica, Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji";
        font-size: clamp(1rem, 2.5vw, 2rem);
        -webkit-font-smoothing: antialiased;
        font-style: normal;
        font-weight: 400;
        letter-spacing: -0.0025em;
        line-height: 1.4;
        min-height: 100dvh;
        place-items: center;
        text-rendering: optimizeLegibility;
        -webkit-text-size-adjust: 100%;
      }

      #error-description {
        fill: #d30001;
      }

      #error-id {
        fill: #f0eff0;
      }

      @media (prefers-color-scheme: dark) {
        body {
          background: #101010;
          color: #e0e0e0;
        }

        #error-description {
          fill: #FF6161;
        }

        #error-id {
          fill: #2c2c2c;
        }
      }

      a {
        color: inherit;
        font-weight: 700;
        text-decoration: underline;
        text-underline-offset: 0.0925em;
      }

      b, strong {
        font-weight: 700;
      }

      i, em {
        font-style: italic;
      }

      main {
        display: grid;
        gap: 1em;
        padding: 2em;
        place-items: center;
        text-align: center;
      }

      main header {
        width: min(100%, 12em);
      }

      main header svg {
        height: auto;
        max-width: 100%;
        width: 100%;
      }

      main article {
        width: min(100%, 30em);
      }

      main article p {
        font-size: 75%;
      }

      main article br {
        display: none;

        @media(min-width: 48em) {
          display: inline;
        }
      }

    </style>

  </head>

  <body>

    <!-- This file lives in public/406-unsupported-browser.html -->

    <main>
      <header>
        <svg height="172" viewBox="0 0 480 172" width="480" xmlns="http://www.w3.org/2000/svg"><path d="m124.48 3.00509-45.6889 100.02991h26.2239v-28.1168h38.119v28.1168h21.628v35.145h-21.628v30.82h-37.308v-30.82h-72.1833v-31.901l50.2851-103.27391zm115.583 168.69891c-40.822 0-64.884-35.146-64.884-85.7015 0-50.5554 24.062-85.700907 64.884-85.700907 40.823 0 64.884 35.145507 64.884 85.700907 0 50.5555-24.061 85.7015-64.884 85.7015zm0-133.2831c-17.572 0-22.709 21.8984-22.709 47.5816 0 25.6835 5.137 47.5815 22.709 47.5815 17.303 0 22.71-21.898 22.71-47.5815 0-25.6832-5.407-47.5816-22.71-47.5816zm202.906 9.7326h-41.093c-2.433-7.2994-7.84-12.4361-17.302-12.4361-16.221 0-25.413 17.5728-25.954 34.8752v1.3517c5.137-7.0291 16.221-12.4361 30.82-12.4361 33.524 0 54.881 24.0612 54.881 53.7998 0 33.253-23.791 58.396-61.64 58.396-21.628 0-39.741-10.003-50.825-27.576-9.733-14.599-13.788-32.442-13.788-54.3406 0-51.9072 24.331-89.485807 66.236-89.485807 32.712 0 53.258 18.654107 58.665 47.851907zm-82.727 66.2355c0 13.247 9.463 22.439 22.71 22.439 12.977 0 22.439-9.192 22.439-22.439 0-13.517-9.462-22.7091-22.439-22.7091-13.247 0-22.71 9.1921-22.71 22.7091z" id="error-id"/><path d="m100.761 68.9967v34.0033h-7.1991l-14.2326-19.8814v19.8814h-8.5839v-34.0033h8.307l13.125 18.7184v-18.7184zm28.454 21.5428c0 7.6978-5.15 13.0145-12.737 13.0145-7.532 0-12.738-5.3167-12.738-13.0145s5.206-13.0143 12.738-13.0143c7.587 0 12.737 5.3165 12.737 13.0143zm-8.529 0c0-3.4336-1.495-5.8703-4.208-5.8703-2.659 0-4.154 2.4367-4.154 5.8703s1.495 5.8149 4.154 5.8149c2.713 0 4.208-2.3813 4.208-5.8149zm13.185 3.8766v-9.5807h-3.655v-6.7564h3.655v-6.8671h8.584v6.8671h5.205v6.7564h-5.205v8.307c0 1.9383.941 2.769 2.658 2.769.941 0 1.994-.2216 2.769-.5538v7.3654c-.997.443-2.88.775-4.818.775-5.87 0-9.193-2.769-9.193-9.0819zm39.02-25.4194h9.083l12.958 34.0033h-9.027l-2.436-6.5902h-12.35l-2.381 6.5902h-8.806zm4.431 10.5222-3.489 9.5807h6.978zm17.44 11.0206c0-7.6978 5.095-13.0143 12.572-13.0143 6.701 0 10.854 3.9874 11.574 9.8023h-8.418c-.221-1.4953-1.384-2.6029-3.156-2.6029-2.437 0-3.988 2.2706-3.988 5.8149s1.551 5.7595 3.988 5.7595c1.772 0 2.935-1.0522 3.156-2.5475h8.418c-.72 5.7596-4.873 9.8025-11.574 9.8025-7.477 0-12.572-5.3167-12.572-13.0145zm25.676 0c0-7.6978 5.095-13.0143 12.572-13.0143 6.701 0 10.854 3.9874 11.574 9.8023h-8.418c-.221-1.4953-1.384-2.6029-3.156-2.6029-2.437 0-3.988 2.2706-3.988 5.8149s1.551 5.7595 3.988 5.7595c1.772 0 2.935-1.0522 3.156-2.5475h8.418c-.72 5.7596-4.873 9.8025-11.574 9.8025-7.477 0-12.572-5.3167-12.572-13.0145zm42.013 3.7658h8.031c-.887 5.7597-5.206 9.2487-11.686 9.2487-7.642 0-12.682-5.2613-12.682-13.0145 0-7.6978 5.317-13.0143 12.516-13.0143 7.643 0 11.962 5.095 11.962 12.5159v2.1598h-16.115c.277 2.9905 1.827 4.5965 4.319 4.5965 1.773 0 3.157-.7753 3.655-2.4921zm-3.821-10.0237c-2.049 0-3.433 1.2737-3.987 3.5997h7.532c-.111-2.0491-1.385-3.5997-3.545-3.5997zm23.4 16.7244v10.799h-8.694v-33.726h8.694v1.9937c1.163-1.3291 3.6-2.5475 6.148-2.5475 7.199 0 11.131 5.8703 11.131 13.0143 0 7.0886-3.932 13.0145-11.131 13.0145-2.548 0-4.985-1.219-6.148-2.548zm0-13.7893v6.5902c.665 1.3845 2.16 2.326 3.822 2.326 2.99 0 4.762-2.3814 4.762-5.5934s-1.772-5.6488-4.762-5.6488c-1.717 0-3.157.9969-3.822 2.326zm21.892 7.1994v-9.5807h-3.655v-6.7564h3.655v-6.8671h8.584v6.8671h5.206v6.7564h-5.206v8.307c0 1.9383.941 2.769 2.658 2.769.942 0 1.994-.2216 2.769-.5538v7.3654c-.997.443-2.88.775-4.818.775-5.87 0-9.193-2.769-9.193-9.0819zm39.458 8.5839h-8.363v-1.274c-.83.831-3.322 1.717-5.981 1.717-4.928 0-9.082-2.769-9.082-8.0301 0-4.818 4.154-7.9193 9.581-7.9193 2.049 0 4.486.6646 5.482 1.3845v-1.606c0-1.606-.941-2.9905-3.045-2.9905-1.606 0-2.548.7199-2.936 1.8275h-8.196c.72-4.8181 4.984-8.6393 11.408-8.6393 7.089 0 11.132 3.7659 11.132 10.2453zm-8.363-6.9779v-1.4399c-.553-1.0522-2.049-1.7167-3.655-1.7167-1.716 0-3.433.7199-3.433 2.3813 0 1.7168 1.717 2.4367 3.433 2.4367 1.606 0 3.102-.6645 3.655-1.6614zm20.742 4.9839v1.994h-8.694v-35.997h8.694v13.0697c1.163-1.3291 3.6-2.5475 6.148-2.5475 7.199 0 11.131 5.8149 11.131 13.0143s-3.932 13.0145-11.131 13.0145c-2.548 0-4.985-1.219-6.148-2.548zm0-13.7893v6.5902c.665 1.3845 2.105 2.326 3.822 2.326 2.99 0 4.762-2.3814 4.762-5.5934s-1.772-5.6488-4.762-5.6488c-1.662 0-3.157.9969-3.822 2.326zm28.759-20.2137v35.997h-8.695v-35.997zm19.172 27.3023h8.03c-.886 5.7597-5.206 9.2487-11.685 9.2487-7.643 0-12.682-5.2613-12.682-13.0145 0-7.6978 5.316-13.0143 12.516-13.0143 7.642 0 11.962 5.095 11.962 12.5159v2.1598h-16.116c.277 2.9905 1.828 4.5965 4.32 4.5965 1.772 0 3.157-.7753 3.655-2.4921zm-3.821-10.0237c-2.049 0-3.434 1.2737-3.988 3.5997h7.532c-.111-2.0491-1.384-3.5997-3.544-3.5997z" id="error-description"/></svg>
      </header>
      <article>
        <p><strong>Your browser is not supported.</strong><br> Please upgrade your browser to continue.</p>
      </article>
    </main>

  </body>

</html>
```

`public/422.html`:

```html
<!doctype html>

<html lang="en">

  <head>

    <title>The change you wanted was rejected (422 Unprocessable Entity)</title>

    <meta charset="utf-8">
    <meta name="viewport" content="initial-scale=1, width=device-width">
    <meta name="robots" content="noindex, nofollow">

    <style>

      *, *::before, *::after {
        box-sizing: border-box;
      }

      * {
        margin: 0;
      }

      html {
        font-size: 16px;
      }

      body {
        background: #FFF;
        color: #261B23;
        display: grid;
        font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, Aptos, Roboto, "Segoe UI", "Helvetica Neue", Helvetica, Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji";
        font-size: clamp(1rem, 2.5vw, 2rem);
        -webkit-font-smoothing: antialiased;
        font-style: normal;
        font-weight: 400;
        letter-spacing: -0.0025em;
        line-height: 1.4;
        min-height: 100dvh;
        place-items: center;
        text-rendering: optimizeLegibility;
        -webkit-text-size-adjust: 100%;
      }

      #error-description {
        fill: #d30001;
      }

      #error-id {
        fill: #f0eff0;
      }

      @media (prefers-color-scheme: dark) {
        body {
          background: #101010;
          color: #e0e0e0;
        }

        #error-description {
          fill: #FF6161;
        }

        #error-id {
          fill: #2c2c2c;
        }
      }

      a {
        color: inherit;
        font-weight: 700;
        text-decoration: underline;
        text-underline-offset: 0.0925em;
      }

      b, strong {
        font-weight: 700;
      }

      i, em {
        font-style: italic;
      }

      main {
        display: grid;
        gap: 1em;
        padding: 2em;
        place-items: center;
        text-align: center;
      }

      main header {
        width: min(100%, 12em);
      }

      main header svg {
        height: auto;
        max-width: 100%;
        width: 100%;
      }

      main article {
        width: min(100%, 30em);
      }

      main article p {
        font-size: 75%;
      }

      main article br {
        display: none;

        @media(min-width: 48em) {
          display: inline;
        }
      }

    </style>

  </head>

  <body>

    <!-- This file lives in public/422.html -->

    <main>
      <header>
        <svg height="172" viewBox="0 0 480 172" width="480" xmlns="http://www.w3.org/2000/svg"><path d="m124.48 3.00509-45.6889 100.02991h26.2239v-28.1168h38.119v28.1168h21.628v35.145h-21.628v30.82h-37.308v-30.82h-72.1833v-31.901l50.2851-103.27391zm130.453 51.63681c0-8.9215-6.218-15.4099-15.681-15.4099-10.273 0-15.95 7.5698-16.491 16.4913h-44.608c3.244-30.8199 25.683-55.421707 61.099-55.421707 36.498 0 59.477 20.816907 59.477 51.636807 0 21.3577-14.869 36.7676-31.901 52.7186l-27.305 27.035h59.747v37.308h-120.306v-27.846l57.044-56.7736c11.084-11.8954 18.925-20.0059 18.925-29.7385zm140.455 0c0-8.9215-6.218-15.4099-15.68-15.4099-10.274 0-15.951 7.5698-16.492 16.4913h-44.608c3.245-30.8199 25.684-55.421707 61.1-55.421707 36.497 0 59.477 20.816907 59.477 51.636807 0 21.3577-14.87 36.7676-31.902 52.7186l-27.305 27.035h59.747v37.308h-120.305v-27.846l57.043-56.7736c11.085-11.8954 18.925-20.0059 18.925-29.7385z" id="error-id"/><path d="m19.3936 103.554c-8.9715 0-14.84183-5.0952-14.84183-14.4544v-20.1029h8.86083v19.3276c0 4.8181 2.2706 7.3102 5.981 7.3102 3.6551 0 5.9257-2.4921 5.9257-7.3102v-19.3276h8.8608v20.1583c0 9.3038-5.8149 14.399-14.7865 14.399zm18.734-.554v-24.921h8.6947v2.1598c1.3845-1.5506 3.8212-2.7136 6.701-2.7136 5.538 0 8.8054 3.5997 8.8054 9.1377v16.3371h-8.6393v-14.2327c0-2.049-1.0522-3.5443-3.2674-3.5443-1.7168 0-3.1567.9969-3.5997 2.7136v15.0634zm36.8584-1.994v10.799h-8.6946v-33.726h8.6946v1.9937c1.163-1.3291 3.5997-2.5475 6.1472-2.5475 7.1994 0 11.1314 5.8703 11.1314 13.0143 0 7.0886-3.932 13.0145-11.1314 13.0145-2.5475 0-4.9842-1.219-6.1472-2.548zm0-13.7893v6.5902c.6646 1.3845 2.1599 2.326 3.8213 2.326 2.9905 0 4.7626-2.3814 4.7626-5.5934s-1.7721-5.6488-4.7626-5.6488c-1.7168 0-3.1567.9969-3.8213 2.326zm36.789-9.2485v8.3624c-1.052-.5538-2.215-.7753-3.6-.7753-2.381 0-3.987 1.0522-4.43 2.8244v14.6203h-8.6949v-24.921h8.6949v2.2152c1.218-1.6614 3.156-2.769 5.648-2.769 1.108 0 1.994.2215 2.382.443zm26.769 12.5713c0 7.6978-5.15 13.0145-12.737 13.0145-7.532 0-12.738-5.3167-12.738-13.0145s5.206-13.0143 12.738-13.0143c7.587 0 12.737 5.3165 12.737 13.0143zm-8.528 0c0-3.4336-1.496-5.8703-4.209-5.8703-2.659 0-4.154 2.4367-4.154 5.8703s1.495 5.8149 4.154 5.8149c2.713 0 4.209-2.3813 4.209-5.8149zm10.352 0c0-7.6978 5.095-13.0143 12.571-13.0143 6.701 0 10.855 3.9874 11.574 9.8023h-8.417c-.222-1.4953-1.385-2.6029-3.157-2.6029-2.437 0-3.987 2.2706-3.987 5.8149s1.55 5.7595 3.987 5.7595c1.772 0 2.935-1.0522 3.157-2.5475h8.417c-.719 5.7596-4.873 9.8025-11.574 9.8025-7.476 0-12.571-5.3167-12.571-13.0145zm42.013 3.7658h8.03c-.886 5.7597-5.206 9.2487-11.685 9.2487-7.643 0-12.682-5.2613-12.682-13.0145 0-7.6978 5.316-13.0143 12.516-13.0143 7.642 0 11.962 5.095 11.962 12.5159v2.1598h-16.116c.277 2.9905 1.828 4.5965 4.32 4.5965 1.772 0 3.156-.7753 3.655-2.4921zm-3.821-10.0237c-2.049 0-3.434 1.2737-3.988 3.5997h7.532c-.111-2.0491-1.385-3.5997-3.544-3.5997zm13.428 11.0206h8.473c.387 1.3845 1.606 2.1598 3.156 2.1598 1.44 0 2.548-.5538 2.548-1.7168 0-.9414-.72-1.2737-1.938-1.5506l-4.874-.9969c-4.153-.886-6.867-2.8797-6.867-7.2547 0-5.3165 4.763-8.4178 10.633-8.4178 6.812 0 10.522 3.1567 11.297 8.0855h-8.03c-.277-1.0522-1.052-1.9937-3.046-1.9937-1.273 0-2.326.5538-2.326 1.6614 0 .7753.554 1.163 1.717 1.3845l4.929 1.163c4.541 1.0522 6.978 3.4335 6.978 7.4763 0 5.3168-4.818 8.2518-10.91 8.2518-6.369 0-10.965-2.88-11.74-8.2518zm24.269 0h8.474c.387 1.3845 1.606 2.1598 3.156 2.1598 1.44 0 2.548-.5538 2.548-1.7168 0-.9414-.72-1.2737-1.939-1.5506l-4.873-.9969c-4.154-.886-6.867-2.8797-6.867-7.2547 0-5.3165 4.763-8.4178 10.633-8.4178 6.812 0 10.522 3.1567 11.297 8.0855h-8.03c-.277-1.0522-1.052-1.9937-3.046-1.9937-1.273 0-2.326.5538-2.326 1.6614 0 .7753.554 1.163 1.717 1.3845l4.929 1.163c4.541 1.0522 6.978 3.4335 6.978 7.4763 0 5.3168-4.818 8.2518-10.91 8.2518-6.369 0-10.965-2.88-11.741-8.2518zm47.918 7.6978h-8.363v-1.274c-.831.831-3.323 1.717-5.981 1.717-4.929 0-9.082-2.769-9.082-8.0301 0-4.818 4.153-7.9193 9.581-7.9193 2.049 0 4.485.6646 5.482 1.3845v-1.606c0-1.606-.941-2.9905-3.046-2.9905-1.606 0-2.547.7199-2.935 1.8275h-8.196c.72-4.8181 4.984-8.6393 11.408-8.6393 7.089 0 11.132 3.7659 11.132 10.2453zm-8.363-6.9779v-1.4399c-.554-1.0522-2.049-1.7167-3.655-1.7167-1.717 0-3.434.7199-3.434 2.3813 0 1.7168 1.717 2.4367 3.434 2.4367 1.606 0 3.101-.6645 3.655-1.6614zm20.742 4.9839v1.994h-8.695v-35.997h8.695v13.0697c1.163-1.3291 3.6-2.5475 6.147-2.5475 7.2 0 11.132 5.8149 11.132 13.0143s-3.932 13.0145-11.132 13.0145c-2.547 0-4.984-1.219-6.147-2.548zm0-13.7893v6.5902c.665 1.3845 2.105 2.326 3.821 2.326 2.991 0 4.763-2.3814 4.763-5.5934s-1.772-5.6488-4.763-5.6488c-1.661 0-3.156.9969-3.821 2.326zm28.759-20.2137v35.997h-8.695v-35.997zm19.172 27.3023h8.03c-.886 5.7597-5.206 9.2487-11.685 9.2487-7.643 0-12.682-5.2613-12.682-13.0145 0-7.6978 5.316-13.0143 12.515-13.0143 7.643 0 11.962 5.095 11.962 12.5159v2.1598h-16.115c.277 2.9905 1.827 4.5965 4.32 4.5965 1.772 0 3.156-.7753 3.655-2.4921zm-3.822-10.0237c-2.049 0-3.433 1.2737-3.987 3.5997h7.532c-.111-2.0491-1.385-3.5997-3.545-3.5997zm25.461-15.2849h24.311v7.6424h-15.561v5.3165h14.232v7.4763h-14.232v5.8703h15.561v7.6978h-24.311zm27.942 34.0033v-24.921h8.694v2.1598c1.385-1.5506 3.822-2.7136 6.701-2.7136 5.538 0 8.806 3.5997 8.806 9.1377v16.3371h-8.639v-14.2327c0-2.049-1.053-3.5443-3.268-3.5443-1.717 0-3.157.9969-3.6 2.7136v15.0634zm29.991-8.5839v-9.5807h-3.655v-6.7564h3.655v-6.8671h8.584v6.8671h5.206v6.7564h-5.206v8.307c0 1.9383.941 2.769 2.658 2.769.942 0 1.994-.2216 2.769-.5538v7.3654c-.997.443-2.88.775-4.818.775-5.87 0-9.193-2.769-9.193-9.0819zm26.161-16.3371v24.921h-8.694v-24.921zm.61-6.7564c0 2.8244-2.271 4.652-4.929 4.652s-4.929-1.8276-4.929-4.652c0-2.8797 2.271-4.7073 4.929-4.7073s4.929 1.8276 4.929 4.7073zm5.382 23.0935v-9.5807h-3.655v-6.7564h3.655v-6.8671h8.584v6.8671h5.206v6.7564h-5.206v8.307c0 1.9383.941 2.769 2.658 2.769.941 0 1.994-.2216 2.769-.5538v7.3654c-.997.443-2.88.775-4.818.775-5.87 0-9.193-2.769-9.193-9.0819zm29.22 17.3889h-8.584l3.655-9.414-9.303-24.312h9.026l4.763 14.1773 4.652-14.1773h8.639z" id="error-description"/></svg>
      </header>
      <article>
        <p><strong>The change you wanted was rejected.</strong> Maybe you tried to change something you didn't have access to. If you're the application owner check the logs for more information.</p>
      </article>
    </main>

  </body>

</html>
```

`public/500.html`:

```html
<!doctype html>

<html lang="en">

  <head>

    <title>We're sorry, but something went wrong (500 Internal Server Error)</title>

    <meta charset="utf-8">
    <meta name="viewport" content="initial-scale=1, width=device-width">
    <meta name="robots" content="noindex, nofollow">

    <style>

      *, *::before, *::after {
        box-sizing: border-box;
      }

      * {
        margin: 0;
      }

      html {
        font-size: 16px;
      }

      body {
        background: #FFF;
        color: #261B23;
        display: grid;
        font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, Aptos, Roboto, "Segoe UI", "Helvetica Neue", Helvetica, Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji";
        font-size: clamp(1rem, 2.5vw, 2rem);
        -webkit-font-smoothing: antialiased;
        font-style: normal;
        font-weight: 400;
        letter-spacing: -0.0025em;
        line-height: 1.4;
        min-height: 100dvh;
        place-items: center;
        text-rendering: optimizeLegibility;
        -webkit-text-size-adjust: 100%;
      }

      #error-description {
        fill: #d30001;
      }

      #error-id {
        fill: #f0eff0;
      }

      @media (prefers-color-scheme: dark) {
        body {
          background: #101010;
          color: #e0e0e0;
        }

        #error-description {
          fill: #FF6161;
        }

        #error-id {
          fill: #2c2c2c;
        }
      }

      a {
        color: inherit;
        font-weight: 700;
        text-decoration: underline;
        text-underline-offset: 0.0925em;
      }

      b, strong {
        font-weight: 700;
      }

      i, em {
        font-style: italic;
      }

      main {
        display: grid;
        gap: 1em;
        padding: 2em;
        place-items: center;
        text-align: center;
      }

      main header {
        width: min(100%, 12em);
      }

      main header svg {
        height: auto;
        max-width: 100%;
        width: 100%;
      }

      main article {
        width: min(100%, 30em);
      }

      main article p {
        font-size: 75%;
      }

      main article br {
        display: none;

        @media(min-width: 48em) {
          display: inline;
        }
      }

    </style>

  </head>

  <body>

    <!-- This file lives in public/500.html -->

    <main>
      <header>
        <svg height="172" viewBox="0 0 480 172" width="480" xmlns="http://www.w3.org/2000/svg"><path d="m101.23 93.8427c-8.1103 0-15.4098 3.7849-19.7354 8.3813h-36.2269v-99.21891h103.8143v37.03791h-68.3984v24.8722c5.1366-2.7035 15.1396-5.9477 24.6014-5.9477 35.146 0 56.233 22.7094 56.233 55.4215 0 34.605-23.791 57.315-60.558 57.315-37.8492 0-61.64-22.169-63.8028-55.963h42.9857c1.0814 10.814 9.1919 19.195 21.6281 19.195 11.355 0 19.465-8.381 19.465-20.547 0-11.625-7.299-20.5463-20.006-20.5463zm138.833 77.8613c-40.822 0-64.884-35.146-64.884-85.7015 0-50.5554 24.062-85.700907 64.884-85.700907 40.823 0 64.884 35.145507 64.884 85.700907 0 50.5555-24.061 85.7015-64.884 85.7015zm0-133.2831c-17.572 0-22.709 21.8984-22.709 47.5816 0 25.6835 5.137 47.5815 22.709 47.5815 17.303 0 22.71-21.898 22.71-47.5815 0-25.6832-5.407-47.5816-22.71-47.5816zm140.456 133.2831c-40.823 0-64.884-35.146-64.884-85.7015 0-50.5554 24.061-85.700907 64.884-85.700907 40.822 0 64.884 35.145507 64.884 85.700907 0 50.5555-24.062 85.7015-64.884 85.7015zm0-133.2831c-17.573 0-22.71 21.8984-22.71 47.5816 0 25.6835 5.137 47.5815 22.71 47.5815 17.302 0 22.709-21.898 22.709-47.5815 0-25.6832-5.407-47.5816-22.709-47.5816z" id="error-id"/><path d="m23.1377 68.9967v34.0033h-8.9162v-34.0033zm4.3157 34.0033v-24.921h8.6947v2.1598c1.3845-1.5506 3.8212-2.7136 6.701-2.7136 5.538 0 8.8054 3.5997 8.8054 9.1377v16.3371h-8.6393v-14.2327c0-2.049-1.0522-3.5443-3.2674-3.5443-1.7168 0-3.1567.9969-3.5997 2.7136v15.0634zm29.9913-8.5839v-9.5807h-3.655v-6.7564h3.655v-6.8671h8.5839v6.8671h5.2058v6.7564h-5.2058v8.307c0 1.9383.9415 2.769 2.6583 2.769.9414 0 1.9937-.2216 2.769-.5538v7.3654c-.9969.443-2.8798.775-4.8181.775-5.8703 0-9.1931-2.769-9.1931-9.0819zm32.3666-.1108h8.0301c-.8861 5.7597-5.2057 9.2487-11.6852 9.2487-7.6424 0-12.682-5.2613-12.682-13.0145 0-7.6978 5.3165-13.0143 12.5159-13.0143 7.6424 0 11.9621 5.095 11.9621 12.5159v2.1598h-16.1156c.2769 2.9905 1.8275 4.5965 4.3196 4.5965 1.7722 0 3.1567-.7753 3.6551-2.4921zm-3.8212-10.0237c-2.0491 0-3.4336 1.2737-3.9874 3.5997h7.5317c-.1107-2.0491-1.3845-3.5997-3.5443-3.5997zm31.4299-6.3134v8.3624c-1.052-.5538-2.215-.7753-3.599-.7753-2.382 0-3.988 1.0522-4.431 2.8244v14.6203h-8.694v-24.921h8.694v2.2152c1.219-1.6614 3.157-2.769 5.649-2.769 1.108 0 1.994.2215 2.381.443zm2.949 25.0318v-24.921h8.694v2.1598c1.385-1.5506 3.821-2.7136 6.701-2.7136 5.538 0 8.806 3.5997 8.806 9.1377v16.3371h-8.64v-14.2327c0-2.049-1.052-3.5443-3.267-3.5443-1.717 0-3.157.9969-3.6 2.7136v15.0634zm50.371 0h-8.363v-1.274c-.83.831-3.323 1.717-5.981 1.717-4.929 0-9.082-2.769-9.082-8.0301 0-4.818 4.153-7.9193 9.581-7.9193 2.049 0 4.485.6646 5.482 1.3845v-1.606c0-1.606-.941-2.9905-3.046-2.9905-1.606 0-2.547.7199-2.935 1.8275h-8.196c.72-4.8181 4.984-8.6393 11.408-8.6393 7.089 0 11.132 3.7659 11.132 10.2453zm-8.363-6.9779v-1.4399c-.554-1.0522-2.049-1.7167-3.655-1.7167-1.717 0-3.433.7199-3.433 2.3813 0 1.7168 1.716 2.4367 3.433 2.4367 1.606 0 3.101-.6645 3.655-1.6614zm20.742-29.0191v35.997h-8.694v-35.997zm13.036 25.9178h9.248c.72 2.326 2.714 3.489 5.483 3.489 2.713 0 4.596-1.163 4.596-3.2674 0-1.6061-1.052-2.326-3.212-2.8244l-6.534-1.3845c-4.985-1.1076-8.751-3.7105-8.751-9.47 0-6.6456 5.538-11.0206 13.07-11.0206 8.307 0 13.014 4.5411 13.956 10.4114h-8.695c-.72-1.8829-2.27-3.3228-5.205-3.3228-2.548 0-4.265 1.1076-4.265 2.9905 0 1.4953 1.052 2.326 2.825 2.7137l6.645 1.5506c5.815 1.3845 9.027 4.5412 9.027 9.8023 0 6.9778-5.87 10.9654-13.291 10.9654-8.141 0-13.679-3.9322-14.897-10.6332zm46.509 1.3845h8.031c-.887 5.7597-5.206 9.2487-11.686 9.2487-7.642 0-12.682-5.2613-12.682-13.0145 0-7.6978 5.317-13.0143 12.516-13.0143 7.643 0 11.962 5.095 11.962 12.5159v2.1598h-16.115c.277 2.9905 1.827 4.5965 4.319 4.5965 1.773 0 3.157-.7753 3.655-2.4921zm-3.821-10.0237c-2.049 0-3.433 1.2737-3.987 3.5997h7.532c-.111-2.0491-1.385-3.5997-3.545-3.5997zm31.431-6.3134v8.3624c-1.053-.5538-2.216-.7753-3.6-.7753-2.381 0-3.988 1.0522-4.431 2.8244v14.6203h-8.694v-24.921h8.694v2.2152c1.219-1.6614 3.157-2.769 5.649-2.769 1.108 0 1.994.2215 2.382.443zm18.288 25.0318h-7.809l-9.47-24.921h8.861l4.763 14.288 4.652-14.288h8.528zm25.614-8.6947h8.03c-.886 5.7597-5.206 9.2487-11.685 9.2487-7.642 0-12.682-5.2613-12.682-13.0145 0-7.6978 5.316-13.0143 12.516-13.0143 7.642 0 11.962 5.095 11.962 12.5159v2.1598h-16.116c.277 2.9905 1.828 4.5965 4.32 4.5965 1.772 0 3.157-.7753 3.655-2.4921zm-3.821-10.0237c-2.049 0-3.434 1.2737-3.988 3.5997h7.532c-.111-2.0491-1.384-3.5997-3.544-3.5997zm31.43-6.3134v8.3624c-1.052-.5538-2.215-.7753-3.6-.7753-2.381 0-3.987 1.0522-4.43 2.8244v14.6203h-8.695v-24.921h8.695v2.2152c1.218-1.6614 3.157-2.769 5.649-2.769 1.107 0 1.993.2215 2.381.443zm13.703-8.9715h24.312v7.6424h-15.562v5.3165h14.232v7.4763h-14.232v5.8703h15.562v7.6978h-24.312zm44.667 8.9715v8.3624c-1.052-.5538-2.215-.7753-3.6-.7753-2.381 0-3.987 1.0522-4.43 2.8244v14.6203h-8.695v-24.921h8.695v2.2152c1.218-1.6614 3.156-2.769 5.648-2.769 1.108 0 1.994.2215 2.382.443zm19.673 0v8.3624c-1.053-.5538-2.216-.7753-3.6-.7753-2.381 0-3.987 1.0522-4.43 2.8244v14.6203h-8.695v-24.921h8.695v2.2152c1.218-1.6614 3.156-2.769 5.648-2.769 1.108 0 1.994.2215 2.382.443zm26.769 12.5713c0 7.6978-5.15 13.0145-12.737 13.0145-7.532 0-12.738-5.3167-12.738-13.0145s5.206-13.0143 12.738-13.0143c7.587 0 12.737 5.3165 12.737 13.0143zm-8.529 0c0-3.4336-1.495-5.8703-4.208-5.8703-2.659 0-4.154 2.4367-4.154 5.8703s1.495 5.8149 4.154 5.8149c2.713 0 4.208-2.3813 4.208-5.8149zm28.082-12.5713v8.3624c-1.052-.5538-2.215-.7753-3.6-.7753-2.381 0-3.987 1.0522-4.43 2.8244v14.6203h-8.695v-24.921h8.695v2.2152c1.218-1.6614 3.157-2.769 5.649-2.769 1.107 0 1.993.2215 2.381.443z" id="error-description"/></svg>
      </header>
      <article>
        <p><strong>We're sorry, but something went wrong.</strong><br> If you're the application owner check the logs for more information.</p>
      </article>
    </main>

  </body>

</html>
```

`public/icon.svg`:

```svg
<svg width="512" height="512" xmlns="http://www.w3.org/2000/svg">
  <circle cx="256" cy="256" r="256" fill="red"/>
</svg>
```

`public/robots.txt`:

```txt
# See https://www.robotstxt.org/robotstxt.html for documentation on how to use the robots.txt file
```

`storage/.keep`:

```txt

```

`test/test_helper.rb`:

```rb
ENV["RAILS_ENV"] ||= "test"
require_relative "../config/environment"
require "rails/test_help"

module ActiveSupport
  class TestCase
    # Run tests in parallel with specified workers
    parallelize(workers: :number_of_processors)

    # Setup all fixtures in test/fixtures/*.yml for all tests in alphabetical order.
    fixtures :all

    # Add more helper methods to be used by all tests here...
  end
end
```

`tmp/.keep`:

```txt

```

