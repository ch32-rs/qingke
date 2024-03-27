#!/bin/bash

set -ex

cargo publish

cd qingke-rt/macros && cargo publish

cd qingke-rt && cargo publish
