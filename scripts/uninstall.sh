#!/bin/bash
# ==============================================================================
# Surface Atmos - Uninstallation Script
# ==============================================================================
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

if [ "$EUID" -eq 0 ]; then
    echo -e "${RED}Error: Do NOT run uninstall.sh with sudo. Run it as your regular user:${NC}"
    echo -e "  ./scripts/uninstall.sh"
    exit 1
fi

echo -e "${RED}${BOLD}Uninstalling Surface Atmos...${NC}"

# Stop systemd service
systemctl --user stop surface-atmos.service 2>/dev/null || true
systemctl --user disable surface-atmos.service 2>/dev/null || true
rm -f "$HOME/.config/systemd/user/surface-atmos.service"
systemctl --user daemon-reload

# Remove PipeWire filter config
rm -f "$HOME/.config/pipewire/pipewire.conf.d/surface-atmos.conf"
systemctl --user restart pipewire

# Remove binary
sudo rm -f /usr/local/bin/surface-atmos

echo -e "${GREEN}✓ Surface Atmos has been completely uninstalled.${NC}"
