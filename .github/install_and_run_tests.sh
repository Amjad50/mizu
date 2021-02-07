#! /bin/bash

set -e

cd test_roms
bash ./installer.sh ./tests_data.csv
cd ..

cargo tarpaulin -t 300 -p mizu-core -v -o Xml
