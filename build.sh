#!/bin/bash

rm -rf target/*
mkdir -p target

cd frontend 
docker run --rm -v .:/app --user $(id -u):$(id -g) -w /app node:26 npm i && npm run build

cd ..
mv ./frontend/dist/inari/browser target/static
cd server

docker buildx build -t inari-builder . 
docker run --rm -v .:/app --user $(id -u):$(id -g) -w /app inari-builder cargo build --release

cd ..
mv server/target/release/inari target/

cp server/.env.example target/.env