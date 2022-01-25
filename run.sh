VERSION="$(git log --pretty=format:"%h" -1)"

chmod +x ./bin/$VERSION-linux-x86_64

./bin/$VERSION-linux-x86_64
