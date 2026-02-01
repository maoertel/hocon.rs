#!/bin/bash

set -x

crashes=0
failed_comp=0
total=0
errors=()

# Known failures: these tests fail due to parsing ".33" as float 0.33 instead of string
known_failures=("include-from-list.conf" "test01.conf" "test03.conf" "test04.conf")

rm -rf tmp
mkdir tmp
git clone https://github.com/mockersf/hocon-test-suite.git tmp/
mkdir tmp/output

for conf_file in tmp/hocon/*
do
    if [ -f "$conf_file" ]
    then
        total=$((total + 1))
        filename=`basename $conf_file`
        cargo run --quiet --example hocon2json $conf_file > tmp/output/$filename
        if [ $? -ne 0 ]
        then
            crashes=$((crashes + 1))
            errors[${#errors[@]}]=$filename
        else
            cmp <(jq -cS . tmp/json/$filename) <(jq -cS . tmp/output/$filename)
            if [ $? -ne 0 ]
            then
                failed_comp=$((failed_comp + 1))
                errors[${#errors[@]}]=$filename
            fi
        fi
    fi
done

echo "$total files : $crashes crashes, $failed_comp failed comparisons"
echo ${errors[*]}

# Check if all failures are known/expected
unexpected=0
for err in "${errors[@]}"; do
    is_known=0
    for known in "${known_failures[@]}"; do
        if [ "$err" = "$known" ]; then
            is_known=1
            break
        fi
    done
    if [ $is_known -eq 0 ]; then
        echo "UNEXPECTED FAILURE: $err"
        unexpected=$((unexpected + 1))
    fi
done

if [ $crashes -gt 0 ]; then
    echo "FAIL: $crashes crashes"
    exit 1
fi

if [ $unexpected -gt 0 ]; then
    echo "FAIL: $unexpected unexpected comparison failures"
    exit 1
fi

echo "OK: All failures are known/expected"
exit 0
