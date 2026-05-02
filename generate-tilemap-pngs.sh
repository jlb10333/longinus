find **.json -exec sudo tmxrasterizer -s 160 --show-layer TilesBG {} "{}.tilesbg.png" \; && find **.json -exec sudo tmxrasterizer -s 160 --show-layer Tiles {} "{}.tiles.png" \;
