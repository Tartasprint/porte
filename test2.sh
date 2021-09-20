#!/bin/bash
testing(){
echo "Ok   101"
echo "Nein 101"

for f in $(ls test_suite/test_parsing/y_*)
do
    if cargo run --release --quiet -- $f
    then
        echo "Ok  " $? $f
    else
        echo "Nein" $? $f
    fi
done
for f in $(ls test_suite/test_parsing/n_*)
do
    if cargo run --release --quiet -- $f
    then
        echo "Nein" $? $f
    else
        echo "Ok  " $? $f
    fi
done

for f in $(ls test_suite/test_parsing/i_*)
do
    if cargo run --release --quiet -- $f
    then
        echo "Ok  " $? $f
    else
        echo "Ok  " $? $f
    fi
done
}

testing 2> /dev/null | grep -v "^Ok   [01] .*\$"