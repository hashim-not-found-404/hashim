default: fmt stat

fmt:
    taplo fmt
    cargo sort -w --grouped
    cargo sort-derives --order "Debug,...,Deserialize,Serialize"
    cargo +nightly fmt
    clear

stat:
    clear
    git ls-files "crates/*.rs" | xargs wc -l | tail -1
    git ls-files "*.rs" | xargs wc -l | tail -1
    git ls-files | xargs wc -l | tail -1

    git ls-files "crates/*" | xargs wc -l | awk '$2 != "total" { split($2,p,"/"); d=p[1]"/"p[2]; s[d]+=$1 } END { for (k in s) print k, s[k] }' | sort

    git rev-list --count HEAD

dump: fmt stat
    git ls-files | while read -r f; do file -b --mime-type "$f" | grep -q "^text/" && { echo "=== $f ==="; nl -ba -w5 -s' | ' "$f"; } done > codebase.txt

check:
    clear
    RUSTFLAGS="-A warnings" cargo check --all-targets
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="server"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="client"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="database"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="cache"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="client,ui"

test:
    clear
    RUSTFLAGS="-A warnings" cargo test

warn:
    clear
    cargo clippy --all-targets --all-features -- -W clippy::pedantic

test_cover:
    clear
    cargo tarpaulin --out HTML
    xdg-open /home/hashem/Documents/backup_folder_for_hashem/accounting_app/tarpaulin-report.html

all:
    clear check test warn test_cover


check_p crate_name:
    clear
    RUSTFLAGS="-A warnings" cargo check -p {{crate_name}} --all-targets
    RUSTFLAGS="-A warnings" cargo check -p {{crate_name}} --all-targets --features="server"
    RUSTFLAGS="-A warnings" cargo check -p {{crate_name}} --all-targets --features="client"
    RUSTFLAGS="-A warnings" cargo check -p {{crate_name}} --all-targets --features="database"
    RUSTFLAGS="-A warnings" cargo check -p {{crate_name}} --all-targets --features="cache"
    RUSTFLAGS="-A warnings" cargo check -p {{crate_name}} --all-targets --features="client,ui"

test_p crate_name:
    clear
    RUSTFLAGS="-A warnings" cargo test -p {{crate_name}}

warn_p crate_name:
    clear
    cargo clippy -p {{crate_name}} --all-targets --all-features -- -W clippy::pedantic

test_cover_p crate_name:
    clear
    cargo tarpaulin -p {{crate_name}} --out HTML
    xdg-open /home/hashem/Documents/backup_folder_for_hashem/accounting_app/tarpaulin-report.html

all_p crate_name:
    @just fmt
    @just check_p {{crate_name}}
    @just test_p {{crate_name}}
    @just warn_p {{crate_name}}
    @just test_cover_p {{crate_name}}


new crate_name:
    cargo new crates/{{crate_name}} --lib --vcs none

run_server:
    clear
    RUSTFLAGS="-A warnings" cargo run -p app_root_server

run_client:
    clear
    RUSTFLAGS="-A warnings" dx serve --open --port=8082 --watch=false --keep-names --debug-symbols=true -p app_root_client

udeps:
    clear
    RUSTFLAGS="-A warnings" cargo +nightly udeps --all-targets

run_db:
    clear
    cockroach start-single-node --insecure --store=trush

push_schema:
    clear
    cockroach sql --file crates/app_root_server/schema/tables.sql --insecure
