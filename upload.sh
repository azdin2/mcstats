cargo build --release
rsync target/release/mcstats mc@simpvp.net:./simpvp/world/
