#!/bin/bash
# ==============================================================================
# Surface Atmos - Installation Script
# Microsoft Surface Laptop Studio Dolby Atmos Adaptive DSP Driver
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
echo "    Surface Atmos Installer for Surface Laptop Studio     "
echo "=========================================================="
echo -e "${NC}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ "$EUID" -eq 0 ]; then
    echo -e "${RED}Error: Do NOT run install.sh with sudo. Run it as your regular user:${NC}"
    echo -e "  ./scripts/install.sh"
    echo -e "The script will automatically request sudo permissions only when copying the binary."
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

# 3. Configure PipeWire Filter-Chain
echo -e "${BLUE}[4/5] Configuring PipeWire 20-band Dolby filter-chain...${NC}"
mkdir -p "$HOME/.config/pipewire/pipewire.conf.d"
cp "$SCRIPT_DIR/pipewire/surface-atmos.conf" "$HOME/.config/pipewire/pipewire.conf.d/surface-atmos.conf"

systemctl --user restart pipewire
sleep 1

# Find surface_atmos_sink and set as default
SINK_ID=$(pw-dump Node 2>/dev/null | grep -B 5 '"node.name": "surface_atmos_sink"' | grep '"id":' | head -n 1 | tr -dc '0-9')
if [ -n "$SINK_ID" ]; then
    wpctl set-default "$SINK_ID"
    echo -e "${GREEN}✓ Set 'Surface Omnisonic (Dolby Atmos DSP)' as default audio sink (Node ID: $SINK_ID).${NC}"
else
    echo -e "${CYAN}Note: PipeWire sink created, will be auto-selected on next audio playback.${NC}"
fi

# 4. Enable systemd user service
echo -e "${BLUE}[5/5] Enabling and starting surface-atmos systemd user service...${NC}"
mkdir -p "$HOME/.config/systemd/user"
cp "$SCRIPT_DIR/systemd/surface-atmos.service" "$HOME/.config/systemd/user/surface-atmos.service"

systemctl --user daemon-reload
systemctl --user enable --now surface-atmos.service

echo ""
echo -e "${GREEN}${BOLD}==========================================================${NC}"
echo -e "${GREEN}${BOLD}    Surface Atmos Successfully Installed and Running!     ${NC}"
echo -e "${GREEN}${BOLD}==========================================================${NC}"
echo ""
echo -e "You can control it anytime with:"
echo -e "  ${CYAN}surface-atmos status${NC}           - View current posture & active EQ curve"
echo -e "  ${CYAN}surface-atmos set <mode>${NC}       - Force mode (laptop, stage, studio)"
echo -e "  ${CYAN}surface-atmos profile <name>${NC}   - Switch profile (balanced, detailed, warm)"
echo -e "  ${CYAN}systemctl --user status surface-atmos${NC} - View daemon service status"
echo ""
