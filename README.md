[![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-blue.svg)](https://opensource.org/licenses/MPL-2.0) [![Test Status](https://travis-ci.org/mozilla-services/megaphone.svg?branch=master)](https://travis-ci.org/mozilla-services/megaphone) [![Build Status](https://circleci.com/gh/mozilla-services/megaphone.svg?style=shield&circle-token=074ae89011d1a7601378c41a4351e1e03f1e8177)](https://circleci.com/gh/mozilla-services/megaphone)

# Megaphone
**A rust based real-time update broadcast server for Firefox**

See [API doc](https://docs.google.com/document/d/1Wxqf1a4HDkKgHDIswPmhmdvk8KPoMEh2q6SPhaz4LNE)


***NOTE***: This will require:

 * rust nightly. See [rocket.rs Getting
   Started](https://rocket.rs/guide/getting-started/) for additional steps.
   To set nightly as the default for only megaphone, from the
   megaphone directory run: rustup override set nightly
 * mysql
 * libmysqlclient installed (brew install mysql on macOS, apt-get install
   libmysqlclient-dev on Ubuntu)

Run:
  * export ROCKET_DATABASE_URL=mysql://scott:tiger@mydatabase/megaphone
  * cargo run
