find **.json -exec sudo tmxrasterizer -s 16 --show-layer TilesBG {} "{}.tilesbg.png" \; && find **.json -exec sudo tmxrasterizer -s 16 --show-layer Tiles {} "{}.tiles.png" \;
