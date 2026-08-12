TARGET="./target/release/pbj_wayland"
DEST="/usr/bin/pbj-wayland"
cargo build --release
if [ -f "$DEST" ]; then
  sudo rm $DEST
fi
sudo cp $TARGET $DEST
