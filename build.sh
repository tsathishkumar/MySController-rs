#!/bin/bash -e
# Manually build OpenSSL. The openssl create requires 1.0.2+, but Travis CI
# only includes 1.0.0.
wget https://www.openssl.org/source/openssl-1.1.0h.tar.gz
tar xzf openssl-1.1.0h.tar.gz
cd openssl-1.1.0h
./config --prefix=/usr/local
make >/dev/null
sudo make install >/dev/null
sudo ldconfig
cd ..
sudo apt-get install gcc-4.7-multilib-arm-linux-gnueabihf
rustup target add armv7-unknown-linux-gnueabihf

cargo test
cargo deb --variant=x86_64 --target x86_64-unknown-linux-gnu
cargo deb --variant=armv7 --target armv7-unknown-linux-gnueabihf