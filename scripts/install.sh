#!/bin/bash
# ==============================================================================
# Surface Acoustics - Adaptive Stage & Studio Acoustic Compensator
# Microsoft Surface Laptop Studio Posture Acoustic & Touchpad Fix
# ==============================================================================
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${CYAN}${BOLD}"
echo "=========================================================="
echo "    Surface Acoustics Installer for Surface Laptop Studio "
echo "=========================================================="
echo -e "${NC}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ "$EUID" -eq 0 ]; then
    echo -e "${RED}Error: Do NOT run install.sh with sudo. Run it as your regular user:${NC}"
    echo -e "  ./scripts/install.sh"
    echo -e "The script will automatically request sudo permissions when installing files."
    exit 1
fi

# 1. Dependency checks
echo -e "${BLUE}[1/5] Checking dependencies...${NC}"
for cmd in cargo rustc pipewire pw-cli wpctl; do
    if ! command -v "$cmd" &> /dev/null; then
        echo -e "${RED}Error: Required command '$cmd' is not installed.${NC}"
        exit 1
    fi
done
echo -e "${GREEN}✓ All dependencies found.${NC}"

# 2. Build Rust release binary
echo -e "${BLUE}[2/5] Building surface-atmos release binary...${NC}"
cd "$SCRIPT_DIR"
cargo build --release

echo -e "${BLUE}[3/5] Installing binary to /usr/local/bin/surface-atmos...${NC}"
sudo cp "$SCRIPT_DIR/target/release/surface-atmos" /usr/local/bin/surface-atmos
sudo chmod +x /usr/local/bin/surface-atmos
echo -e "${GREEN}✓ Binary installed.${NC}"

# 3. Configure Stage Trackpad fix (keep trackpad working in Stage mode)
echo -e "${BLUE}[4/5] Configuring Stage Mode trackpad fix and PipeWire Smart Filter...${NC}"
sudo sh -c 'cat << "EOF" > /etc/modprobe.d/surface-tabletsw.conf
# Keep trackpad/touchpad active in Stage/Slate mode on Surface Laptop Studio
options surface_aggregator_tabletsw tablet_mode_in_slate_state=0
EOF'
if [ -w /sys/module/surface_aggregator_tabletsw/parameters/tablet_mode_in_slate_state ]; then
    echo "N" | sudo tee /sys/module/surface_aggregator_tabletsw/parameters/tablet_mode_in_slate_state > /dev/null || true
fi
echo -e "${GREEN}✓ Stage Mode trackpad fix installed.${NC}"

# Configure PipeWire Smart Filter
mkdir -p "$HOME/.config/pipewire/pipewire.conf.d"
cp "$SCRIPT_DIR/pipewire/surface-atmos.conf" "$HOME/.config/pipewire/pipewire.conf.d/surface-atmos.conf"

mkdir -p "$HOME/.config/wireplumber/wireplumber.conf.d"
cp "$SCRIPT_DIR/wireplumber/50-bluez.conf" "$HOME/.config/wireplumber/wireplumber.conf.d/50-bluez.conf"

systemctl --user restart pipewire wireplumber
sleep 1

# 4. Enable systemd user service
echo -e "${BLUE}[5/5] Enabling and starting surface-atmos systemd user service...${NC}"
mkdir -p "$HOME/.config/systemd/user"
cp "$SCRIPT_DIR/systemd/surface-atmos.service" "$HOME/.config/systemd/user/surface-atmos.service"

systemctl --user daemon-reload
systemctl --user enable --now surface-atmos.service

echo ""
echo -e "${GREEN}${BOLD}==========================================================${NC}"
echo -e "${GREEN}${BOLD}    Surface Acoustics Successfully Installed & Running!   ${NC}"
echo -e "${GREEN}${BOLD}==========================================================${NC}"
echo ""
echo -e "Features active:"
echo -e "  ${GREEN}✓${NC} Clean native Built-in Audio in Laptop mode (zero latency, full dynamic range)"
echo -e "  ${GREEN}✓${NC} Smart +4.5 dB High-Shelf treble lift in Stage mode (cuts cleanly through glass)"
echo -e "  ${GREEN}✓${NC} Active touchpad and mouse in Stage mode over the keyboard"
echo -e "  ${GREEN}✓${NC} Zero duplicate sound devices or volume desync in GNOME"
echo ""
echo -e "Commands:"
echo -e "  ${CYAN}surface-atmos status${NC}     - View current posture & active acoustic parameters"
echo -e "  ${CYAN}surface-atmos set <mode>${NC} - Force mode (laptop, stage, studio)"
echo ""
