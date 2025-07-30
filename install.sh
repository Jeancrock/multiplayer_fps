
#!/bin/bash
cargo clean
echo "**********************************************"
echo ""
echo "Installation des prérecquis"
echo ""
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
rustup update
echo ""
sudo apt install libasound2-dev pkg-config
echo ""
sudo apt install libudev-dev
echo ""
echo "Prérecquis installés"
echo ""
echo "**********************************************"
echo ""
echo "Installation des dépendances"
echo ""
cargo build --release
echo ""
echo "Dépendances installées"
echo "**********************************************"
