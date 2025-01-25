sudo apt install build-essential libreadline-dev zlib1g-dev flex bison libxml2-dev libxslt-dev libssl-dev libxml2-utils xsltproc ccache pkg-config

cargo pgrx new pg_jalali_calendar
cargo pgrx package
cargo pgrx install
cargo pgrx run pg16
cargo pgrx run pg17

cargo pgrx test pg16
cargo pgrx test pg17

