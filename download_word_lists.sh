#!/usr/bin/env bash

url_base="https://gist.githubusercontent.com/SachaG/1c87e1e2ef55a8c38fec7793c23821f8/raw/7e3e0d504aa767bc361fc294d97c982f1c2f7839/words"
for n in 2 5 10; do
  echo "Downloading ${n}k word list"
  curl --create-dirs -o "data/words${n}k.json" "${url_base}${n}k.json"
done

megalist="https://raw.githubusercontent.com/martinheidegger/kansai-algo/main/words_alpha.txt"
echo "Downloading megalist (thanks Martin!)"
curl -o "data/words_mega.json" $megalist
