#!/bin/bash

#git pull
#ps -ef | grep "target/release/webscale" | grep -v grep | awk '{print $2}' | xargs kill
#cargo clean
#cargo run --release

while [ 1 ]
do
	git pull
	cargo run --release
done
