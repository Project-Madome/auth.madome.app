VERSION="$(git log --pretty=format:"%h" -1)"
FULL_VERSION="$(git log --pretty=format:"%H" -1)"

mkdir -p "bin/linux-x86_64"

echo "cargo build\n";

OPENSSL_DIR="/usr/local/opt/openssl" cargo build --release --target=x86_64-unknown-linux-musl

if [ $? -ne 0 ]; then
    exit 1
fi

cp ./target/x86_64-unknown-linux-musl/release/madome-auth "./bin/linux-x86_64/${VERSION}"

github-release -v release \
    --user syrflover \
    --repo "auth.madome.app" \
    --target "$FULL_VERSION" \
    --tag "_${VERSION}" \
    --name "$VERSION" \
    --description "$(date "+%Y.%m.%d.%H.%m")" \
    --pre-release

if [ $? -ne 0 ]; then
    echo "\nfailed release";
    exit 1
fi

echo "\nsucceed release\n";

github-release -v upload \
    --user syrflover \
    --repo "auth.madome.app" \
    --tag "_${VERSION}" \
    --name "${VERSION}-linux-x86_64" \
    --file bin/linux-x86_64/$VERSION

if [ $? -ne 0 ]; then
    echo "\nfailed upload to release";
    exit 1
fi

echo "\nsucceed upload to release\n";
