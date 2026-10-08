TARGET="./target/release/pbj_wayland"
DEST="/usr/bin/pbj_wayland"

cargo build --release

sudo install -m 755 -D "$TARGET" "$DEST"
