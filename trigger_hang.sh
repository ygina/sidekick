#!/bin/bash
for i in {1..1000}
do
    echo "Running $i"
    curl http://10.0.1.10:8000/ --max-time 5
done
