#!/bin/bash

zip -qr temp.zip Cargo.toml Cargo.lock src
hlt bot -b temp.zip
rm temp.zip
