default: fmt stat

stat:
    @clear
    git ls-files "crates/accounting_engine/*.rs" | xargs wc -l | tail -1
    git ls-files "crates/*.rs" | xargs wc -l | tail -1
    git ls-files "*.rs" | xargs wc -l | tail -1
    git ls-files | xargs wc -l | tail -1
    git rev-list --count HEAD

fmt:
    @cargo sort -w --grouped && cargo sort-derives --order "Debug,...,Deserialize,Serialize" && cargo +nightly fmt && clear

dump: fmt stat
    @find . -type f \( -name "*.rs" -o -name "*.sql" -o -name "*.toml" \) -not -path "*/target/*" -not -path "*/.git/*" -exec echo "=== {} ===" \; -exec cat {} \; > codebase.txt

check: fmt
    RUSTFLAGS="-A warnings" cargo check --all-targets
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="infrastructure"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="client"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="client,infrastructure,cache"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="client,infrastructure,ui"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="server"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="server,infrastructure,database"

test: fmt
    RUSTFLAGS="-A warnings" cargo test -- --show-output

warn: fmt
    cargo clippy --all-targets --all-features -- -W clippy::pedantic

checkp crate_name: fmt
    RUSTFLAGS="-A warnings" cargo check -p {{crate_name}} --all-targets
    RUSTFLAGS="-A warnings" cargo check -p {{crate_name}} --all-targets --features="infrastructure"
    RUSTFLAGS="-A warnings" cargo check -p {{crate_name}} --all-targets --features="client"
    RUSTFLAGS="-A warnings" cargo check -p {{crate_name}} --all-targets --features="client,infrastructure,cache"
    RUSTFLAGS="-A warnings" cargo check -p {{crate_name}} --all-targets --features="client,infrastructure,ui"
    RUSTFLAGS="-A warnings" cargo check -p {{crate_name}} --all-targets --features="server"
    RUSTFLAGS="-A warnings" cargo check -p {{crate_name}} --all-targets --features="server,infrastructure,database"

testp crate_name: fmt
    RUSTFLAGS="-A warnings" cargo test -p {{crate_name}} -- --show-output

warnp crate_name: fmt
    cargo clippy -p {{crate_name}} --all-targets --all-features -- -W clippy::pedantic

all: fmt check test warn

allp crate_name:
    @just fmt
    @just checkp {{crate_name}}
    @just testp {{crate_name}}
    @just warnp {{crate_name}}
