# beta, stable일 경우에만 릴리즈
# beta, stable 따로 관리할 수 있어야함

SVC=auth

CURRENT_BRANCH="$(git branch --show-current)"
VERSION="$(cat Cargo.toml | grep 'version = ' | head -1 | sed -e 's/version = //' | sed -e 's/\"//g')"

if [ "$CURRENT_BRANCH" = "beta" ]; then
    TARGET="debug"

    cargo build --debug --target=x86_64-unknown-linux-musl
elif [ "$CURRENT_BRANCH" = "stable" ]; then
    TARGET="release"

    cargo build --release --target=x86_64-unknown-linux-musl
else
    echo "can't release from master branch"
    exit 1
fi

if [ $? -ne 0 ]; then
    exit 1
fi

# BIN_DIR="./bin/$TARGET"

# mkdir -p "bin/$TARGET"

# cp ./target/x86_64-unknown-linux-musl/$TARGET/madome-user "$BIN_DIR/$VERSION"

github-release -v release \
    --user Project-Madome \
    --repo "$SVC.madome.app" \
    --tag "${CURRENT_BRANCH}/${VERSION}" \
    --name "Released ${CURRENT_BRANCH}/${VERSION}" \
    --description "$(date "+%Y.%m.%d.%H.%m")" \
    --pre-release

if [ $? -ne 0 ]; then
    echo "failed release"
    exit 1
fi

echo "succeed release\n"

github-release -v upload \
    --user Project-Madome \
    --repo "$SVC.madome.app" \
    --tag "${CURRENT_BRANCH}/${VERSION}" \
    --name "${CURRENT_BRANCH}-$VERSION" \
    --file "./target/x86_64-unknown-linux-musl/$TARGET/madome-$SVC"

if [ $? -ne 0 ]; then
    echo "failed upload"
    exit 1
fi

echo "succeed upload"
