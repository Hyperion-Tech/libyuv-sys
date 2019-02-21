# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --target $TARGET --features "bundled"
    cross build --target $TARGET --features "bundled" --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET --features "bundled"
    cross test --target $TARGET --features "bundled" --release

    cross run --target $TARGET --features "bundled"
    cross run --target $TARGET --features "bundled" --release
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
