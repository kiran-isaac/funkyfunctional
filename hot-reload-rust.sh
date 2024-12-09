while inotifywait -e close_write wasm-lib/src/*; do  npm run rs-build; npm run build; done
