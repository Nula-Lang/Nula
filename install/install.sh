#!/bin/bash
echo "[DOWNLOAD]"
curl -L --fail --show-error --progress-bar "https://github.com/Nula-Lang/Nula/releases/download/v0.1/nula-zig" -o /tmp/nula-zig
curl -L --fail --show-error --progress-bar "https://github.com/Nula-Lang/Nula/releases/download/v0.1/nula-go" -o /tmp/nula-go
curl -L --fail --show-error --progress-bar "https://github.com/Nula-Lang/Nula/releases/download/v0.1/nula" -o /tmp/nula
curl -L --fail --show-error "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula.png" -o /tmp/nula.png
curl -L --fail --show-error --progress-bar "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula-terminal.sh" -o /tmp/nula-terminal.sh
curl -L --fail --show-error "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula-lang.desktop" -o /tmp/nula-lang.desktop
echo "[INFO] Moving and updating file permissions."
sudo mkdir /usr/lib/nula
sudo chmod a+x /tmp/nula-terminal.sh
sudo chmod a+x /tmp/nula
sudo chmod a+x /tmp/nula-go
sudo chmod a+x /tmp/nula-zig
sudo mv /tmp/nula /usr/bin/
sudo mv /tmp/nula-zig /usr/bin/
sudo mv /tmp/nula-go /usr/bin/
sudo mv /tmp/nula-lang.desktop
sudo mv /tmp/nula-terminal.sh /usr/lib/nula/
sudo mv /tmp/nula.png /usr/share/icons/
sudo mv /tmp/nula-lang.desktop /usr/share/applications/
echo "[INFO] The nula programming language has been installed."
echo "[INFO] Run the nula command or nula application in the terminal."
sleep 10
bash
nula ?
