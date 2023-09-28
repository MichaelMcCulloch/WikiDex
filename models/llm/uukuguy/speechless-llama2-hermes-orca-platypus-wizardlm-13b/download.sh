#!/bin/bash

file="download.txt"

while IFS= read -r url
do
  wget -q "$url" &
done < "$file"

wait
