[![License: MPL 2.0][mpl-svg]][mpl] [![Test Status][travis-badge]][travis] [![Build Status][circleci-badge]][circleci]

# Megaphone

## What is it?

Megaphone is an internal Mozilla system providing global broadcasts for Firefox.

Traditionally Firefox has polled multiple services at different frequencies (e.g. every 24 hours) to check for updates. Megaphone serves as an alternative, notifying user agents of new updates in near real time (within 5 minutes) over the [WebPush WebSocket protocol].

This enables Firefox to:

* [Revoke HTTPS Certificates] or [malicious extensions] immediately after security incidents occur
* Update quicker (Firefox itself, [tracking protection lists], or [general settings])
* Provide faster turn-around/feedback loops for studies/experiments ([Shield])

All via one unified, simpler client-side service that doesn't require polling.

This repository provides a Rust based Megaphone endpoint (API). Broadcasts are sent to the Megaphone endpoint. The [autopush-rs service] polls the endpoint as the source of truth for current broadcasts, ultimately delivering them to clients.

Also see the [API doc].


## Requirements

 * Rust nightly as specified in the `rust-toolchain` file (recognized by cargo) in the root of the project (recognized by cargo).
 * MySQL 5.7 (or compatible)
 * libmysqlclient (brew install mysql on macOS, apt-get install libmysqlclient-dev on Ubuntu)

 * *For running the docker image locally: docker-compose v1.21.0 or later

## Setting Up

1) [Install Rust]

2) Create a `megaphone` user/database

3) Run:

  $ export ROCKET_DATABASE_URL=mysql://scott:tiger@mydatabase/megaphone
  $ cargo run

## Running the Docker Image

1) [Install docker-compose]

2) From a dedicated screen (or tmux window)

$ `docker-compose up`

This will create two intertwined docker images:

***db_1*** - the database image. This image is a local test image. The database can be accessed via `mysql -umegaphone -ptest -h localhost --port 4306 megaphone`.

***app_1*** - the **megaphone** application. This is accessible via port 8000.


# API

Megaphone is normally called via a HTTP interface using Authorized calls. Responses are generally JSON objects with appropriate HTTP status codes to indicate success/failure.

## Authorization

All calls to Megaphone (minus the Dockerflow Status Checks) require authorization. Authorization is specified by the `Authorization` header via Bearer tokens specified in the application's configurtion.

e.g.

```
export ROCKET_BROADCASTER_AUTH={test="foobar"}
export ROCKET_READER_AUTH={autopush="quux"}
```

The *test* broadcaster would specify:

```
Authorization: Bearer foobar
```

The *autopush* reader would specify:

```
Authorization: Bearer quux
```


## PUT /v1/broadcasts/< broadcaster_id > /< bchannel_id >

Broadcast a new version.

The body of the PUT request becomes the new version value.

A special version value of "____NOP____" (the string "NOP" prefixed and suffixed by four underscores) signals a "No Operation" to clients: that no action should take place, effectively overwriting and cancelling any pending version update.

The return value is a JSON structure including the HTTP status of the result: successful results either being a `201` code for newly created broadcasts or `200` for an update to an existing broadcast.

```javascript
{
   "code": 200
}
```


## GET /v1/broadcasts

Read the current broadcasts.

The return value is a JSON structure including the HTTP status of the result and a `broadcasts` object consisting of broadcastIDs and their current versions.

```javascript
{
   "code": 200,
   "broadcasts": {
      "test/broadcast1": "v3",
      "test/broadcast2": "v0"
   }
}
```

## Dockerflow Status Checks:

## GET /\_\_heartbeat__

Return the status of the server.

This call is only used for server status checks.


## GET /\_\_lbheartbeat__

Return a light weight status check (200 OK).

This call is only used for the Load Balancer's check.


## GET /\_\_version__

Return a JSON response of the version information of the server.


[mpl-svg]: https://img.shields.io/badge/License-MPL%202.0-blue.svg
[mpl]: https://opensource.org/licenses/MPL-2.0
[travis-badge]: https://travis-ci.org/mozilla-services/megaphone.svg?branch=master
[travis]: https://travis-ci.org/mozilla-services/megaphone
[circleci-badge]: https://circleci.com/gh/mozilla-services/megaphone.svg?style=shield&circle-token=074ae89011d1a7601378c41a4351e1e03f1e8177
[circleci]: https://circleci.com/gh/mozilla-services/megaphone

[WebPush WebSocket protocol]: https://mozilla-push-service.readthedocs.io/en/latest/design/#simplepush-protocol
[revoke HTTPS Certificates]: https://blog.mozilla.org/security/2015/03/03/revoking-intermediate-certificates-introducing-onecrl/
[malicious extensions]: https://wiki.mozilla.org/Blocklisting
[tracking protection lists]: https://wiki.mozilla.org/Security/Safe_Browsing
[general settings]: https://wiki.mozilla.org/Firefox/RemoteSettings
[Shield]: https://wiki.mozilla.org/Firefox/Shield/Shield_Studies
[autopush-rs service]: https://github.com/mozilla-services/autopush-rs
[API doc]: https://docs.google.com/document/d/1Wxqf1a4HDkKgHDIswPmhmdvk8KPoMEh2q6SPhaz4LNE

[Install Rust]: https://rustup.rs/
[Install docker-compose]: https://docs.docker.com/compose/install/
