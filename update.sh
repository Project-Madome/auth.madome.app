# beta, stable 따로 구분해서 받아올 수 있어야함

SVC=auth

CURRENT_BRANCH="$(git branch --show-current)"
VERSION="$(cat Cargo.toml | grep 'version = ' | head -1 | sed -e 's/version = //' | sed -e 's/\"//g')"

if [ "$CURRENT_BRANCH" = "beta" ] || [ "$CURRENT_BRANCH" = "stable" ]; then
    git pull

    if [ $? -ne 0 ]; then
        echo "failed git pull";
        exit 1
    fi

    github-release -v download \
        --user Project-Madome \
        --repo "$SVC.madome.app" \
        --tag "${CURRENT_BRANCH}/${VERSION}" \
        --name "${CURRENT_BRANCH}-$VERSION"

    if [ $? -ne 0 ]; then
        echo "failed download from release"
        exit 1
    fi

    mkdir -p ./bin/$CURRENT_BRANCH

    mv "./${CURRENT_BRANCH}-${VERSION}" "./bin/$CURRENT_BRANCH/$VERSION"

    echo "succeed download"
else
    echo "not release branch"
fi
