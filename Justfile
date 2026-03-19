# Justfile for bbpipelinediag

rust_image := 'rust:1.85.0'
target := `rustc -vV | sed -n 's|host: ||p'`
os_family := os_family()
archive_type := if os_family == "windows" { "zip" } else { "tarball" }
os := os()
arch := arch()
version := `toml get Cargo.toml package.version --raw`
archive_name := "bbpipelinediag-" + version + "-" + target
msg := "Unknown error"
binary_name := if os_family == "windows" { "bbpipelinediag.exe" } else { "bbpipelinediag" }

default: build

die:
    @echo "Error: {{ msg }}"

clean:
    rm -rf target dist

build:
    cargo build --release --target {{ target }}

check:
    cargo check
    cargo clippy -- -D warnings
    cargo fmt -- --check

fmt:
    cargo fmt

test:
    cargo test

archive-tarball:
    mkdir -p dist/{{ target }}
    tar czf dist/{{ target }}/{{ archive_name }}.tar.gz -C target/{{ target }}/release/ {{ binary_name }}

archive-zip:
    mkdir -p dist/{{ target }}
    zip dist/{{ target }}/{{ archive_name }}.zip target/{{ target }}/release/{{ binary_name }}

archive: build
    just target={{ target }} archive-{{ archive_type }}

build-all: build-mac-m1 build-mac-x86 build-linux-amd64 build-linux-arm64

build-mac-m1:
    just target=aarch64-apple-darwin archive

build-mac-x86:
    just target=x86_64-apple-darwin archive

build-linux-amd64:
    docker run --rm --platform linux/amd64 --user "$(id -u)":"$(id -g)" -v "$PWD":/usr/src/myapp -w /usr/src/myapp \
        {{ rust_image }} sh -c "cargo install just toml-cli && just archive"

build-linux-arm64:
    docker run --rm --platform linux/arm64 --user "$(id -u)":"$(id -g)" -v "$PWD":/usr/src/myapp -w /usr/src/myapp \
        {{ rust_image }} sh -c "cargo install just toml-cli && just archive"
