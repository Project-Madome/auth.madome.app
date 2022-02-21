SVC=auth

CURRENT_BRANCH="$(git branch --show-current)"

VERSION="$(cat Cargo.toml | grep 'version = ' | head -1 | sed -e 's/version = //' | sed -e 's/\"//g')"

BIN="./bin/${CURRENT_BRANCH}/$VERSION"

if [ "$CURRENT_BRANCH" != "stable" ] && [ "$CURRENT_BRANCH" != "beta" ]; then
    cargo build --target=x86_64-unknown-linux-musl

    if [ $? -ne 0 ]; then
        exit 1
    fi

    kubectl apply -f k8s_node_port.yml

    VERSION="latest"

    BIN="./target/x86_64-unknown-linux-musl/debug/madome-$SVC"
else
    if [ ! -f $BIN ]; then
        echo "binary file download"
        ./update.sh
    fi

    if [ ! -f $BIN ]; then
        echo "binary file does not released or not found"
        exit 1
    fi

    kubectl apply -f k8s_cluster_ip.yml

    if [ ! -f $BIN ]; then
        exit 1
    fi

    # e.g. stable-0.1.1
    VERSION="${CURRENT_BRANCH}-$VERSION"
fi

chmod +x $BIN

docker build --build-arg BINARY_FILE="$BIN" --tag "madome-user:$VERSION" .

if [ $? -ne 0 ]; then
    echo "failed docker build"
    exit 1
fi

cat k8s_deployment.yml | \
sed -e "s/{VERSION}/$VERSION/g" | \
kubectl apply -f -

if [ $? -ne 0 ]; then
    echo "failed apply kubectl"
    exit 1
fi

if [ "$CURRENT_BRANCH" != "stable" ]; then
    kubectl rollout restart deployment/madome-$SVC
fi
