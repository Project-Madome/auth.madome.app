git pull;

if [ $? -ne 0 ]; then
    echo "failed git pull\n";
    exit 1
fi

VERSION="$(git log --pretty=format:"%h" -1)"

github-release -v download \
    --user syrflover \
    --repo auth.madome.app \
    --tag "_${VERSION}" \
    --name "${VERSION}-linux-x86_64"

if [ $? -ne 0 ]; then
    echo "\nfailed download from release\n";
    exit 1
fi

mkdir -p ./bin

mv "./${VERSION}-linux-x86_64" ./bin

echo "\nsucceed download\n"