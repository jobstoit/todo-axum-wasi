FROM scratch
COPY target/wasm32-wasi/release/fragmends.wasm /fragmends.wasm
ENTRYPOINT ["/fragmends.wasm"]
