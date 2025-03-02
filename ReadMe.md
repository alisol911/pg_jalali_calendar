sudo apt install build-essential libreadline-dev zlib1g-dev flex bison libxml2-dev libxslt-dev libssl-dev libxml2-utils xsltproc ccache pkg-config clang
sudo apt install postgresql-server-dev-17

cargo install --locked cargo-pgrx
cargo pgrx init --pg17 /usr/bin/pg_config

cargo pgrx new pg_jalali_calendar
cargo pgrx package
cargo pgrx install
cargo pgrx run pg17

cargo pgrx test pg17

