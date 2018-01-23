# This script takes care of testing your crate

set -euxo pipefail

main() {
    cross check --target $TARGET
}

main
