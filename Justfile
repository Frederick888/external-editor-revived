set shell := ["bash", "+u", "-c"]

default:
    cargo fmt -- --check
    cargo clippy --locked
    cargo clippy --locked --tests
    cargo test --quiet
    cargo build

lint:
    cargo fmt -- --check
    cargo clippy --locked -- -D warnings
    cargo clippy --locked --tests -- -D warnings

macos_uni:
    set -e
    cargo build --target=x86_64-apple-darwin --locked --release
    cargo build --target=aarch64-apple-darwin --locked --release
    lipo -create -output target/external-editor-revived target/x86_64-apple-darwin/release/external-editor-revived target/aarch64-apple-darwin/release/external-editor-revived

update:
    UPDATED_CRATES="$(cargo update 2>&1 | sed -n 's/^\s*Updating \(.*->.*\)/\1/p')"; \
        if [[ -z "$UPDATED_CRATES" ]]; then \
            printf 'Already up to date\n'; \
        else \
            cargo test || exit 1; \
            git add Cargo.lock; \
            printf 'chore(host): Upgrade dependencies\n\n%s\n' "$UPDATED_CRATES" | git commit -F -; \
        fi
    @printf 'Running cargo outdated\n'
    cargo outdated -R

release version:
    set -e
    @if [[ "{{version}}" == v* ]]; then printf 'Must not have v-prefix\n'; exit 1; fi
    # changelog
    if [[ "{{version}}" != *"-"* ]]; then \
        last_tag="$(git tag -l --sort version:refname | grep -v -- - | tail -n1)"; \
        clog --from="$last_tag" --setversion=v{{version}} -o ./CHANGELOG.md; \
        git add ./CHANGELOG.md; \
    fi
    # host
    sed 's/^version = ".*"$/version = "{{version}}"/' -i ./Cargo.toml
    cargo update -p external-editor-revived
    git add ./Cargo.toml ./Cargo.lock
    just lint
    cargo test
    cargo build
    # extension
    jq --indent 4 '.version = "{{version}}"' ./extension/manifest.json | sponge ./extension/manifest.json
    jq '.version = "{{version}}"' ./extension/package.json | sponge ./extension/package.json
    git add ./extension/manifest.json ./extension/package.json
    # commit and tag
    git status
    git diff --exit-code
    git commit -m 'chore: Bump version to {{version}}'
    git tag v{{version}}

# vim: set filetype=just :
