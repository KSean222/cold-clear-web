(
    export RUSTFLAGS="--cfg=web_sys_unstable_apis"
    wasm-pack build "${1:-"--dev"}" --target no-modules --out-name main
)
cp -r ./static/* ./pkg
