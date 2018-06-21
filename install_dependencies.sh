#!/bin/bash -e
sudo apt-get install libudev-dev

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