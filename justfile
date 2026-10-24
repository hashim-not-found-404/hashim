default: fmt check

fmt:
    taplo fmt                                                       > /tmp/fmt.log 2>&1 || { cat /tmp/fmt.log; exit 1; }
    cargo sort -w --grouped                                         > /tmp/fmt.log 2>&1 || { cat /tmp/fmt.log; exit 1; }
    cargo sort-derives --order "Debug,...,Deserialize,Serialize"    > /tmp/fmt.log 2>&1 || { cat /tmp/fmt.log; exit 1; }
    cargo +nightly fmt                                              > /tmp/fmt.log 2>&1 || { cat /tmp/fmt.log; exit 1; }

stat:
    git ls-files "crates/*.rs" | xargs wc -l | tail -1
    git ls-files | xargs wc -l | tail -1
    git ls-files "crates/*" | xargs wc -l | awk '$2 != "total" { split($2,p,"/"); d=p[1]"/"p[2]; s[d]+=$1 } END { for (k in s) print k, s[k] }' | sort
    git rev-list --count HEAD

dump:
    #!/usr/bin/env bash
    set -euo pipefail
    commit="$(git rev-parse HEAD)"
    echo "commit: $commit" > codebase.txt
    git ls-tree -r "$commit" | while IFS=$'\t' read -r meta path; do
        set -- $meta
        type="$2"
        hash="$3"
        [ "$type" = "blob" ] || continue
        if git cat-file blob "$hash" | file -b --mime-type - | grep -q '^text/'; then
            echo "=== $path ==="
            git cat-file blob "$hash" | nl -ba -w5 -s' | '
        fi
    done >> codebase.txt

# Annotated diffs since the last `just dump`.
# Layout:
#   ### commit <hash>
#   ###    <subject>
#     --- file: <path>
#      123 + | added line
#      124 - | removed line
diffs:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f codebase.txt ]; then
        echo "error: codebase.txt not found; run 'just dump' first" >&2
        exit 1
    fi
    base="$(head -n1 codebase.txt | sed -E 's/^commit:[[:space:]]*//')"
    if ! git rev-parse --verify "${base}^{commit}" >/dev/null 2>&1; then
        echo "error: could not resolve commit '$base' from codebase.txt" >&2
        exit 1
    fi
    head="$(git rev-parse HEAD)"
    {
        echo "=== diffs from $base to $head ==="

        # ---- committed diffs -------------------------------------------------
        git log --reverse -p --format='COMMIT %H%nSUBJECT %s' "$base"..HEAD |
        awk '
            /^COMMIT /  { hash = $2; next }
            /^SUBJECT / { subj = substr($0, 9); next }
            /^diff --git/ {
                if (cur_commit != hash) {
                    print ""
                    print "### commit " hash
                    print "###    " subj
                    cur_commit = hash
                }
                file = $3; sub(/^a\//, "", file)
                print ""
                print "  --- file: " file
                next
            }
            /^@@ /              { match($0, /\+[0-9]+/); lineno = substr($0, RSTART+1, RLENGTH-1) + 0; next }
            /^\+/ && !/^\+\+\+/ { printf "    %5d + | %s\n", lineno, substr($0,2); lineno++; next }
            /^-/  && !/^---/    { printf "    %5d - | %s\n", lineno, substr($0,2); next }
            /^ /                { lineno++; next }
        '

        # ---- uncommitted changes --------------------------------------------
        u="$(git diff HEAD)"
        if [ -n "$u" ]; then
            echo
            echo "### uncommitted changes"
            printf '%s\n' "$u" | awk '
                /^diff --git/ {
                    file = $3; sub(/^a\//, "", file)
                    print ""
                    print "  --- file: " file
                    next
                }
                /^@@ /              { match($0, /\+[0-9]+/); lineno = substr($0, RSTART+1, RLENGTH-1) + 0; next }
                /^\+/ && !/^\+\+\+/ { printf "    %5d + | %s\n", lineno, substr($0,2); lineno++; next }
                /^-/  && !/^---/    { printf "    %5d - | %s\n", lineno, substr($0,2); next }
                /^ /                { lineno++; next }
            '
        fi

        # ---- untracked files ------------------------------------------------
        untracked="$(git ls-files --others --exclude-standard -- ':!diffs.txt' ':!codebase.txt')"
        if [ -n "$untracked" ]; then
            echo
            echo "### new files (untracked)"
            while IFS= read -r f; do
                if file -b --mime-type "$f" | grep -q "^text/"; then
                    echo ""
                    echo "  --- file: $f"
                    nl -ba -w5 -s' | ' "$f"
                fi
            done <<< "$untracked"
        fi
    } > diffs.txt
    echo "wrote diffs.txt ($(wc -l < diffs.txt) lines, $(du -h diffs.txt | cut -f1))"

# Write the FULL contents of every file changed since the last `just dump`
# into `changed.txt`. Covers commits since the stored base, uncommitted
# edits, and untracked text files. Same `=== path ===` / `nl` format as `dump`.
changed:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f codebase.txt ]; then
        echo "error: codebase.txt not found; run 'just dump' first" >&2
        exit 1
    fi
    base="$(head -n1 codebase.txt | sed -E 's/^commit:[[:space:]]*//')"
    if ! git rev-parse --verify "${base}^{commit}" >/dev/null 2>&1; then
        echo "error: could not resolve commit '$base' from codebase.txt" >&2
        exit 1
    fi
    head="$(git rev-parse HEAD)"

    {
        # Files touched by commits made since the last dump
        git diff --name-only "$base".."$head"
        # Tracked files with staged/unstaged edits against HEAD
        git diff --name-only HEAD
        # New untracked files (respecting .gitignore; skip dump artifacts)
        git ls-files --others --exclude-standard -- ':!codebase.txt' ':!diffs.txt' ':!changed.txt'
    } | sort -u | while IFS= read -r f; do
        [ -f "$f" ] || continue
        if file -b --mime-type "$f" | grep -q '^text/'; then
            echo ""
            echo "=== $f ==="
            nl -ba -w5 -s' | ' "$f"
        fi
    done > changed.txt

    echo "wrote changed.txt ($(wc -l < changed.txt) lines, $(du -h changed.txt | cut -f1))"

###################################################################################################

check:
    RUSTFLAGS="-A warnings" cargo check --all-targets
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="server"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="client"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="database"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="cache"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="client,ui"
    RUSTFLAGS="-A warnings" cargo check --all-targets --features="client,mcp"
    RUSTFLAGS="-A warnings" cargo check -p app_root_client --target wasm32-unknown-unknown

test:
    RUSTFLAGS="-A warnings" cargo test

warn:
    cargo clippy --all-targets --all-features -- -W clippy::pedantic

test_cover:
    cargo tarpaulin --out HTML
    xdg-open /home/hashem/Documents/backup_folder_for_hashem/accounting_app/tarpaulin-report.html

all: check test warn test_cover

###################################################################################################

test_p crate_name:
    RUSTFLAGS="-A warnings" cargo test -p {{crate_name}}

warn_p crate_name:
    cargo clippy -p {{crate_name}} --all-targets --all-features -- -W clippy::pedantic

test_cover_p crate_name:
    cargo tarpaulin -p {{crate_name}} --out HTML
    xdg-open /home/hashem/Documents/backup_folder_for_hashem/accounting_app/tarpaulin-report.html

all_p crate_name:
    @just fmt
    @just test_p {{crate_name}}
    @just warn_p {{crate_name}}
    @just test_cover_p {{crate_name}}

###################################################################################################

udeps:
    RUSTFLAGS="-A warnings" cargo +nightly udeps --all-targets
    RUSTFLAGS="-A warnings" cargo +nightly udeps --all-targets --features="server"
    RUSTFLAGS="-A warnings" cargo +nightly udeps --all-targets --features="client"
    RUSTFLAGS="-A warnings" cargo +nightly udeps --all-targets --features="database"
    RUSTFLAGS="-A warnings" cargo +nightly udeps --all-targets --features="cache"
    RUSTFLAGS="-A warnings" cargo +nightly udeps --all-targets --features="client,ui"
    RUSTFLAGS="-A warnings" cargo +nightly udeps --all-targets --features="client,mcp"
    RUSTFLAGS="-A warnings" cargo +nightly udeps -p app_root_client --target wasm32-unknown-unknown

new crate_name:
    cargo new crates/{{crate_name}} --lib --vcs none

run_server:
    RUSTFLAGS="-A warnings" cargo run -p app_root_server

run_client platform:
    RUSTFLAGS="-A warnings" dx serve --{{platform}} --open --port=8082 --watch=false --keep-names --debug-symbols=true --hot-reload=false -p app_root_client

run_db:
    cockroach start-single-node --insecure --store=trush

push_schema:
    cockroach sql --file crates/app_root_server/schema/tables.sql --insecure

mcp_inspect:
    npx @modelcontextprotocol/inspector

###################################################################################################
# branch workflow
###################################################################################################

# Create a new branch off the current HEAD and push it to origin.
# Usage: just new_branch my_feature
new_branch branch_name:
    git checkout -b {{branch_name}}
    git push -u origin {{branch_name}}

# Merge the current branch into `target` with a merge bubble, push, and
# return to the branch you started on.
# Usage (while on `home_forms`): just merge_branch main
merge_branch target:
    #!/usr/bin/env bash
    set -euo pipefail
    source="$(git rev-parse --abbrev-ref HEAD)"
    if [ "$source" = "{{target}}" ]; then
        echo "error: already on '{{target}}'; nothing to merge" >&2
        exit 1
    fi
    git fetch origin
    git checkout "{{target}}"
    git pull --ff-only origin "{{target}}"
    git merge --no-ff "$source" -m "Merge branch '$source'"
    git push origin "{{target}}"
    git checkout "$source"
    echo "merged '$source' into '{{target}}' and pushed"

# Delete a branch locally and on origin. Safe: uses `-d`, which refuses
# to delete a branch that hasn't been merged.
# Usage: just delete_branch my_feature
delete_branch branch_name:
    git branch -d {{branch_name}}
    git push origin --delete {{branch_name}}

# Show local branches with their upstream tracking state.
branches:
    git branch -vv
