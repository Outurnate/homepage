build:
    rm -rf ./output/*
    RUST_BACKTRACE=full ./bin/site-generator --content ./content/ --dist ./output/ --url https://outurnate.com/ --d2 ./bin/d2 --font-regular ./fonts/FiraSans-Regular.ttf --font-italic ./fonts/FiraSans-Italic.ttf --font-bold ./fonts/FiraSans-Bold.ttf --font-semibold ./fonts/FiraSans-SemiBold.ttf --font-monospace ./fonts/FiraCode-VF.ttf
    ./bin/minify --html-keep-default-attrvals --html-keep-document-tags --html-keep-end-tags --html-keep-quotes --inplace --json-keep-numbers --recursive --verbose ./output/
    du -h -d1 ./output/

retool:
    cargo build --release --target=x86_64-unknown-linux-musl
    cp ./target/x86_64-unknown-linux-musl/release/site-generator ./bin/

watch:
    #!/usr/bin/env bash
    trap 'kill $BGPID; exit' INT
    python -m http.server -d ./output/ 8000 &
    BGPID=$!
    inotifywait --monitor --recursive --event modify --event move --event create --event delete --format "%w%f" ./content/ |\
    while read -r path
    do
        echo "$path changed"
        echo "Skipping $(timeout 1 cat | wc -l) further changes"
        time {{ just_executable() }} build
        echo "Up to date"
    done
