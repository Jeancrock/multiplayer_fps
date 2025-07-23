
cargo clean
echo "**********************************************"
echo ""
echo "Installation des prérecquis"
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
cargo build
echo ""
echo "Dépendances installées"
echo "**********************************************"
