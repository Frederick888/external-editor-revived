set shell := ["bash", "+u", "-c"]

lint:
    cargo fmt -- --check
    cargo clippy -- -D warnings

update:
    UPDATED_CRATES="$(cargo update 2>&1 | sed -n 's/^\s*Updating \(.*->.*\)/\1/p')"; \
        if [[ -z "$UPDATED_CRATES" ]]; then \
            printf 'Already up to date\n'; \
        else \
            cargo test || exit 1; \
            git add Cargo.lock; \
            printf 'chore: Upgrade dependencies\n\n%s\n' "$UPDATED_CRATES" | git commit -F -; \
        fi
    @printf 'Running cargo outdated\n'
    cargo outdated -R

# vim: set filetype=just :
