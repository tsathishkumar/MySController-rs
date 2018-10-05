#!/bin/bash -e
# Manually build OpenSSL. The openssl crate requires 1.0.2+, but Travis CI
# only includes 1.0.0.
wget https://www.openssl.org/source/openssl-1.1.0h.tar.gz
tar xzf openssl-1.1.0h.tar.gz
cd openssl-1.1.0h
./config --prefix=/usr/local
make >/dev/null
sudo make install >/dev/null
sudo ldconfig
cd ..
sudo apt-get install -qq gcc-arm-linux-gnueabihf
rustup target add armv7-unknown-linux-gnueabihf

cargo test
cargo build --release --target x86_64-unknown-linux-gnu
# cd /tmp

# wget https://www.openssl.org/source/openssl-1.0.1t.tar.gz
# tar xzf openssl-1.0.1t.tar.gz
# export MACHINE=armv7
# export ARCH=arm
# export CC=arm-linux-gnueabihf-gcc
# cd openssl-1.0.1t && ./config shared && make && cd -

export OPENSSL_LIB_DIR=./openssl-1.1.0h
export OPENSSL_INCLUDE_DIR=./openssl-1.1.0h/include
PKG_CONFIG_ALLOW_CROSS=1 cargo build --release --target armv7-unknown-linux-gnueabihf
cargo deb --no-build --variant=x86_64 
cargo deb --no-build --variant=armv7