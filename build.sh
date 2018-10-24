#!/bin/bash -e
# Manually build OpenSSL. The openssl crate requires 1.0.2+, but Travis CI
# only includes 1.0.0.
# wget https://www.openssl.org/source/openssl-1.1.1.tar.gz
# tar xzf openssl-1.1.1.tar.gz
# cd openssl-1.1.1
# ./config --prefix=/usr/local shared
# make >/dev/null
# sudo make install >/dev/null
# sudo ldconfig
# cd ..
export PKG_CONFIG_ALLOW_CROSS=1
sudo ls -ltra /root
cross test --target armv7-unknown-linux-gnueabihf
cross build --target armv7-unknown-linux-gnueabihf
cargo deb --target x86_64-unknown-linux-gnu
# cargo deb --no-build --variant=armv7