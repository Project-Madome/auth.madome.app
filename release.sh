if [ "$(git branch --show-current)" = "release" ]; then
    VERSION="$(cat Cargo.toml | grep 'version = ' | head -1 | sed -e 's/version = //' | sed -e 's/\"//g')"
    # FULL_VERSION="$(git log --pretty=format:"%H" -1)"

    mkdir -p "bin/linux-x86_64"

    echo "cargo build\n"

    cargo build --release --target=x86_64-unknown-linux-musl

    if [ $? -ne 0 ]; then
        exit 1
    fi

    cp ./target/x86_64-unknown-linux-musl/release/madome-auth "./bin/linux-x86_64/$VERSION"

    github-release -v release \
        --user syrflover \
        --repo "auth.madome.app" \
        # --target "$FULL_VERSION" \
        --tag "v$VERSION" \
        --name "Released v$VERSION" \
        --description "$(date "+%Y.%m.%d.%H.%m")" \
        --pre-release

    if [ $? -ne 0 ]; then
        echo "failed release"
        exit 1
    fi

    echo "succeed release\n"

    github-release -v upload \
        --user syrflover \
        --repo "auth.madome.app" \
        --tag "v$VERSION" \
        --name "madome-auth-linux-x86_64" \
        --file ./bin/linux-x86_64/$VERSION

    if [ $? -ne 0 ]; then
        echo "failed upload"
        exit 1
    fi

    echo "succeed upload"
else
    echo "not release branch"
fi

