# Megaphone
**A rust based real-time update broadcast server for Firefox**

See [API doc](https://docs.google.com/document/d/1Wxqf1a4HDkKgHDIswPmhmdvk8KPoMEh2q6SPhaz4LNE)


***NOTE***: This will require:

 * rust nightly. See [rocket.rs Getting
   Started](https://rocket.rs/guide/getting-started/) for additional steps.
 * mysql
 * libmysqlclient installed (brew install mysql on macOS, apt-get install
   libmysqlclient-dev on Ubuntu)
 * diesel cli: (cargo install diesel_cli --no-default-features
   --features mysql)

Run:
  * export ROCKET_DATABASE_URL=mysql://scott:tiger@mydatabase/megaphone
  * $ diesel setup --database-url $ROCKET_DATABASE_URL
