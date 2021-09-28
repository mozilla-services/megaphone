<a name="0.2.0"></a>
## 0.2.0 (2021-09-28)


#### Bug Fixes

*   oops, improve the health check (#97) ([d26e6d23](https://github.com/mozilla-services/megaphone/commit/d26e6d23d7543d0c2925318907b6eed839b513f2), closes [#96](https://github.com/mozilla-services/megaphone/issues/96))
*   Add libcurl4 to Dockerfile ([ab19b4f7](https://github.com/mozilla-services/megaphone/commit/ab19b4f7e382e6002150218f9fffeaf556a79267), closes [#94](https://github.com/mozilla-services/megaphone/issues/94))

#### Features

*   Add metrics ([9fdb5973](https://github.com/mozilla-services/megaphone/commit/9fdb5973a05d49bfd3a49b8fc2c6ebc4f4c00182), closes [#83](https://github.com/mozilla-services/megaphone/issues/83))
*   add sentry error reporting ([ec2a104c](https://github.com/mozilla-services/megaphone/commit/ec2a104ccdc65749a457ab4e153f19842e18f593), closes [#70](https://github.com/mozilla-services/megaphone/issues/70))

#### Chore

*   Base to rocket "0.4" ([67f4b98f](https://github.com/mozilla-services/megaphone/commit/67f4b98fd4f4cc236d0ca3c5405c709e2ff182eb), closes [#84](https://github.com/mozilla-services/megaphone/issues/84))
*   update circleci to use new docker auth (#85) ([0813565e](https://github.com/mozilla-services/megaphone/commit/0813565eae8d6df8db816ada519dde320416762c))
*   Update Docker to rust 1.45 ([58d0cc4a](https://github.com/mozilla-services/megaphone/commit/58d0cc4a12e9a3a11d5ddd1d0d142f354e29baf4))
*   add a badge for the matrix channel ([40cacd24](https://github.com/mozilla-services/megaphone/commit/40cacd24a6ec3737906eb1cc96fbc0c1c7a83ca2))



<a name="0.1.6"></a>
## 0.1.6 (2020-02-29)


#### Chore

*   upgrade debian stretch -> buster ([85cbe9e8](https://github.com/mozilla-services/megaphone/commit/85cbe9e8cf8b27361519ed7bb1cf00d283e90aa5))
*   fix cargo install ([34b9809c](https://github.com/mozilla-services/megaphone/commit/34b9809cadce9cbc289781615f524bf844bdd44d))
*   cargo fmt ([1fd2e8e3](https://github.com/mozilla-services/megaphone/commit/1fd2e8e30e8276374a5d559192080b615684d806))
*   cargo fix --edition-idioms ([e2836bd7](https://github.com/mozilla-services/megaphone/commit/e2836bd70fe91ac0e06ec2bfb4fbbeda07107ddd))
*   cargo fix --edition ([372dc903](https://github.com/mozilla-services/megaphone/commit/372dc903e736d2aa37523ba0791eb62ee9008273))
*   update deps ([cb2a11e5](https://github.com/mozilla-services/megaphone/commit/cb2a11e5a1daa0f5012142cf094eabc989c3991b), closes [#76](https://github.com/mozilla-services/megaphone/issues/76))
*   update deps per cargo audit ([e6585acc](https://github.com/mozilla-services/megaphone/commit/e6585acccc2c8044e22c1246b98e95fa68fc6d97), closes [#74](https://github.com/mozilla-services/megaphone/issues/74))
*   utilize a rust-toolchain file ([a4b6e1f1](https://github.com/mozilla-services/megaphone/commit/a4b6e1f1155a13bc32dbcad1d06c04303666ea02))

#### Features

*   update dependencies ([1aa4aa1d](https://github.com/mozilla-services/megaphone/commit/1aa4aa1d233df749fbced6db720b0d19ee21df59), closes [#62](https://github.com/mozilla-services/megaphone/issues/62))
*   improve the README ([e1aa5077](https://github.com/mozilla-services/megaphone/commit/e1aa5077069f6b5a66eb04b7791c6510336cbc0f), closes [#59](https://github.com/mozilla-services/megaphone/issues/59))

#### Bug Fixes

*   get_bool now returns ConfigError::Missing ([644d33f4](https://github.com/mozilla-services/megaphone/commit/644d33f4c7d8575670ddff300925fe9f6dfb8387), closes [#67](https://github.com/mozilla-services/megaphone/issues/67))
*   propagate db pool errors into HandlerErrors ([3156643a](https://github.com/mozilla-services/megaphone/commit/3156643a338dbd99c42ce848a5c6e549a2f67324), closes [#59](https://github.com/mozilla-services/megaphone/issues/59))

#### Refactor

*   utilize slog_derive ([16b0922f](https://github.com/mozilla-services/megaphone/commit/16b0922f19cc4d9eb5e6c78243bc4cf0b653895a), closes [#64](https://github.com/mozilla-services/megaphone/issues/64))
*   prefer regular slog macro names ([1e6b61df](https://github.com/mozilla-services/megaphone/commit/1e6b61df8500e52b1296193e10ba7039d0b1dc62))



<a name="0.1.5"></a>
## 0.1.5 (2018-08-21)


#### Features

*   hacky support of ROCKET_LOG=off ([584f758b](https://github.com/mozilla-services/megaphone/commit/584f758bfe9ca3aec83d39848b3d441baa22b092), closes [#54](https://github.com/mozilla-services/megaphone/issues/54))



<a name="0.1.4"></a>
## 0.1.4 (2018-06-27)


#### Chore

*   include /app/version.json per dockerflow ([d5978e3c](https://github.com/mozilla-services/megaphone/commit/d5978e3c9a475208965537c21d88681277876890))

#### Features

*   warn log for ACL related errors ([109476a5](https://github.com/mozilla-services/megaphone/commit/109476a5e6313de761a593b5e4dcbd18a93f34c4), closes [#48](https://github.com/mozilla-services/megaphone/issues/48))
*   render a unique errno code per error ([57025884](https://github.com/mozilla-services/megaphone/commit/57025884993b16dfdf779d414ccda9afa353dc78), closes [#46](https://github.com/mozilla-services/megaphone/issues/46))
*   validate the PUT input ([6fbf572d](https://github.com/mozilla-services/megaphone/commit/6fbf572d685a32c7b60fbb0b7531422fbbd0781d), closes [#24](https://github.com/mozilla-services/megaphone/issues/24))
*   add a docker compose setup ([2c1aa8f1](https://github.com/mozilla-services/megaphone/commit/2c1aa8f1441d8b66eb23c8370c18806738fdc674), closes [#43](https://github.com/mozilla-services/megaphone/issues/43))
*   create a logging setup via slog/slog-mozlog-json ([c7353f7e](https://github.com/mozilla-services/megaphone/commit/c7353f7e167a34aa7617cc23ac9489f395d77ff4), closes [#9](https://github.com/mozilla-services/megaphone/issues/9))



<a name="0.1.3"></a>
## 0.1.3 (2018-05-17)


#### Bug Fixes

*   lheartbeat -> lbheartbeat ([2d8bf644](https://github.com/mozilla-services/megaphone/commit/2d8bf644ba65bd6aaa65a70510a33db6dfaaac8e), closes [#40](https://github.com/mozilla-services/megaphone/issues/40))



<a name="0.1.2"></a>
## 0.1.2 (2018-05-16)


#### Features

*   upgrade to rocket 0.3.10 ([45d3b004](https://github.com/mozilla-services/megaphone/commit/45d3b0047fa5843049518ea7df780eb442bd89f1), closes [#36](https://github.com/mozilla-services/megaphone/issues/36))



<a name="0.1.1"></a>
## 0.1.1 (2018-04-06)


#### Chore

*   install openssh-client git & git on docker-in-docker ([f81d556a](https://github.com/mozilla-services/megaphone/commit/f81d556a415d7e775e358373df22e125d0c37406))



<a name="0.1.0"></a>
## 0.1.0 (2018-04-05)


#### Bug Fixes

*   use mysql, not postgres ([0130713f](https://github.com/mozilla-services/megaphone/commit/0130713fac7f30b986043f7d0ea74e40a234cecb))

#### Features

*   peg Docker build to rust nightly-2018-04-04 ([f17d4924](https://github.com/mozilla-services/megaphone/commit/f17d4924e2f0a74026540e47cbc3a669acc071f9), closes [#25](https://github.com/mozilla-services/megaphone/issues/25))
*   peg builds w/ a Cargo.lock ([e359991f](https://github.com/mozilla-services/megaphone/commit/e359991f235b7f5c1e9eee5e1ef0f51d2419d3a2), closes [#29](https://github.com/mozilla-services/megaphone/issues/29))
*   add a .clog.toml and CONTRIBUTING.md ([f2a33e7a](https://github.com/mozilla-services/megaphone/commit/f2a33e7a578a67da593aa6491396e0a184d101c1), closes [#27](https://github.com/mozilla-services/megaphone/issues/27))
*   switch REPLACE INTO -> INSERT ON DUPLICATE KEY UPDATE ([f09af1c0](https://github.com/mozilla-services/megaphone/commit/f09af1c0cdf807deae7863ad034b3e2caf5c0ec5), closes [#19](https://github.com/mozilla-services/megaphone/issues/19))
*   return WWW-Authenticate on 401s ([348f3e91](https://github.com/mozilla-services/megaphone/commit/348f3e919380c5d3f7ea8913c6d1fbced593feb9), closes [#21](https://github.com/mozilla-services/megaphone/issues/21))
*   use diesel's embedded migrations ([64e00c83](https://github.com/mozilla-services/megaphone/commit/64e00c83e43542ff86dfdcbd8e674395b858ce6e), closes [#17](https://github.com/mozilla-services/megaphone/issues/17))
*   cleanup Config usage ([e5ddf43e](https://github.com/mozilla-services/megaphone/commit/e5ddf43ef1ac910b1202fc6c687fd1d828fafb0c), closes [#16](https://github.com/mozilla-services/megaphone/issues/16))
*   handle auth via Bearer tokens ([91e28568](https://github.com/mozilla-services/megaphone/commit/91e28568b2b28f51642b29d649c23b3f71b3e767), closes [#7](https://github.com/mozilla-services/megaphone/issues/7))
*   add Dockerflow styled health/version checks ([cb616611](https://github.com/mozilla-services/megaphone/commit/cb61661172906fed34ce0b1ba12ee7796fde61f4), closes [#11](https://github.com/mozilla-services/megaphone/issues/11))
*   add a Dockerfile based on debian stretch-slim ([e66b6cc9](https://github.com/mozilla-services/megaphone/commit/e66b6cc98905823ab36b808bf1b8d06c6da74a02), closes [#12](https://github.com/mozilla-services/megaphone/issues/12))
*   initial prototype from the wip branch ([9e6f9b28](https://github.com/mozilla-services/megaphone/commit/9e6f9b289b12df6e5f10ac4a0f1d07ffce5b2777))



<a name="0.1.0"></a>
## 0.1.0 (2018-04-05)


#### Bug Fixes

*   use mysql, not postgres ([0130713f](https://github.com/mozilla-services/megaphone/commit/0130713fac7f30b986043f7d0ea74e40a234cecb))

#### Features

*   peg Docker build to rust nightly-2018-04-04 ([f17d4924](https://github.com/mozilla-services/megaphone/commit/f17d4924e2f0a74026540e47cbc3a669acc071f9), closes [#25](https://github.com/mozilla-services/megaphone/issues/25))
*   peg builds w/ a Cargo.lock ([e359991f](https://github.com/mozilla-services/megaphone/commit/e359991f235b7f5c1e9eee5e1ef0f51d2419d3a2), closes [#29](https://github.com/mozilla-services/megaphone/issues/29))
*   add a .clog.toml and CONTRIBUTING.md ([f2a33e7a](https://github.com/mozilla-services/megaphone/commit/f2a33e7a578a67da593aa6491396e0a184d101c1), closes [#27](https://github.com/mozilla-services/megaphone/issues/27))
*   switch REPLACE INTO -> INSERT ON DUPLICATE KEY UPDATE ([f09af1c0](https://github.com/mozilla-services/megaphone/commit/f09af1c0cdf807deae7863ad034b3e2caf5c0ec5), closes [#19](https://github.com/mozilla-services/megaphone/issues/19))
*   return WWW-Authenticate on 401s ([348f3e91](https://github.com/mozilla-services/megaphone/commit/348f3e919380c5d3f7ea8913c6d1fbced593feb9), closes [#21](https://github.com/mozilla-services/megaphone/issues/21))
*   use diesel's embedded migrations ([64e00c83](https://github.com/mozilla-services/megaphone/commit/64e00c83e43542ff86dfdcbd8e674395b858ce6e), closes [#17](https://github.com/mozilla-services/megaphone/issues/17))
*   cleanup Config usage ([e5ddf43e](https://github.com/mozilla-services/megaphone/commit/e5ddf43ef1ac910b1202fc6c687fd1d828fafb0c), closes [#16](https://github.com/mozilla-services/megaphone/issues/16))
*   handle auth via Bearer tokens ([91e28568](https://github.com/mozilla-services/megaphone/commit/91e28568b2b28f51642b29d649c23b3f71b3e767), closes [#7](https://github.com/mozilla-services/megaphone/issues/7))
*   add Dockerflow styled health/version checks ([cb616611](https://github.com/mozilla-services/megaphone/commit/cb61661172906fed34ce0b1ba12ee7796fde61f4), closes [#11](https://github.com/mozilla-services/megaphone/issues/11))
*   add a Dockerfile based on debian stretch-slim ([e66b6cc9](https://github.com/mozilla-services/megaphone/commit/e66b6cc98905823ab36b808bf1b8d06c6da74a02), closes [#12](https://github.com/mozilla-services/megaphone/issues/12))
*   initial prototype from the wip branch ([9e6f9b28](https://github.com/mozilla-services/megaphone/commit/9e6f9b289b12df6e5f10ac4a0f1d07ffce5b2777))



