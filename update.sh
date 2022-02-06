if [ "$(git branch)" = "* release" ]; then
    git pull

    if [ $? -ne 0 ]; then
        echo "failed git pull\n";
        exit 1
    fi

    VERSION="$(cat Cargo.toml | grep 'version = ' | head -1 | sed -e 's/version = //' | sed -e 's/\"//g')"

    github-release -v download \
        --user syrflover \
        --repo auth.madome.app \
        --tag "v${VERSION}" \
        --name "madodme-auth-linux-x86_64"

    if [ $? -ne 0 ]; then
        echo "\nfailed download from release\n"
        exit 1
    fi

    mkdir -p ./bin

    mv "./madome-auth-linux-x86_64" "./bin/linux-x86_64/$VERSION"

    echo "\nsucceed download\n"
else
    echo "not release branch"
fi
