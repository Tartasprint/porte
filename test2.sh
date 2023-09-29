#!/bin/bash
testing(){
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

if testing 2> /dev/null | grep -v "^Ok   [01] .*\$"
then
    echo "Atleast a test didn't pass."
else
    echo "Everything is ok."
fi