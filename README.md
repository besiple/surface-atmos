# 🎧 Surface Atmos

**Hardware-tuned Dolby Atmos DSP reproduction and adaptive posture acoustic driver for Microsoft Surface Laptop Studio on Linux.**

[![Platform](https://img.shields.io/badge/Platform-Arch%20Linux%20%7C%20Fedora%20%7C%20Ubuntu-blue.svg)](https://github.com/linux-surface/linux-surface)
[![Rust](https://img.shields.io/badge/Language-Rust%202021-orange.svg)](https://www.rust-lang.org/)
[![Audio](https://img.shields.io/badge/Audio-PipeWire%20Filter--Chain-green.svg)](https://pipewire.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## 🌟 Overview

The **Microsoft Surface Laptop Studio (1 & 2)** features a unique **Quad Omnisonic speaker system** (dual downward/keyboard-firing woofers + dual side-firing tweeters) and a flexible 3-position hinge:
1. **Laptop Mode (`laptop`)**: Traditional clamshell, open keyboard.
2. **Stage Mode (`slate`)**: The screen pulls forward over the keyboard onto magnetic latches.
3. **Studio Mode (`tablet`)**: The screen folds completely flat as a drawing canvas.

Under Windows, official Dolby Atmos drivers monitor the Surface posture sensor in real-time. When the glass screen covers the keyboard in **Stage** or **Studio** modes, it physically muffles the woofers. Dolby dynamically compensates for this acoustic obstruction by lifting attenuated treble, boosting vocal clarity, and widening the stereo soundstage.

**`surface-atmos` brings this full adaptive experience to Linux.**

---

## ✨ Features

- **20-band IEQ (Intelligent Equalizer):** Exact Dolby Atmos center frequencies (`47, 141, 234, 328, 469, 656, 844, 1031, 1313, 1688, 2250, 3000, 3750, 4688, 5813, 7125, 9000, 11250, 13875, 19688 Hz`).
- **Asymmetric Hardware Speaker Correction:** Factory calibration curves for Left (`ch_00`) and Right (`ch_01`) channels extracted from official Windows DAX3 tuning files (`DEV_0274_SUBSYS_10EC121C`).
- **Adaptive Posture Acoustic Compensation:**
  - 💻 **Laptop:** Open, balanced acoustic profile.
  - 🎬 **Stage Mode (`slate`):** High-frequency lift (+3.5 dB across 3–10 kHz) to penetrate through the screen covering the keyboard, plus dialogue enhancement.
  - 🎨 **Studio Mode (`tablet`):** Desk table resonance damping (-2 dB at 234 Hz) and acoustic vent compensation (+4 dB on highs, expanded stereo width).
- **Glitch-Free Smooth Crossfade:** When switching modes, parameters glide over 250ms via smoothstep S-curve interpolation without audio dropouts or clicks.
- **PipeWire Native Filter-Chain:** Uses PipeWire's built-in SIMD biquad processors in the real-time audio thread for ultra-low latency and ~0.05% CPU usage.
- **Desktop Notifications:** Displays a neat desktop notification when screen posture changes.
- **CLI & Systemd:** Full command-line interface and automatic launch as a `systemd --user` service.

---

## 🚀 Quick Installation

Clone and install with one command:

```bash
git clone https://github.com/besiple/surface-atmos.git
cd surface-atmos
./scripts/install.sh
```

*(You will be prompted for `sudo` to install the binary to `/usr/local/bin/surface-atmos`).*

---

## 🛠️ Usage

### Check Status
Display hardware sensor status, active posture mode, and the current 20-band gain curve:

```bash
surface-atmos status
```

### Manually Override Mode
Manually test or force a specific mode:

```bash
surface-atmos set laptop
surface-atmos set stage
surface-atmos set studio
```

### Switch EQ Profile Target
Switch between official Dolby Atmos curve targets:

```bash
surface-atmos profile balanced   # Balanced default
surface-atmos profile detailed   # Enhanced clarity & crisp highs
surface-atmos profile warm       # Rich, relaxed sound
```

---

## ⚙️ Configuration

Settings are saved in `~/.config/surface-atmos/config.toml`:

```toml
# Default EQ profile: "balanced", "detailed", or "warm"
profile = "balanced"

# Crossfade slew duration in milliseconds (default: 250)
transition_duration_ms = 250

# Slew step interval in milliseconds (default: 25)
step_interval_ms = 25

# Show desktop notification on posture change (default: false)
enable_notifications = false

# PipeWire sink name
sink_name = "surface_atmos_sink"
```

---

## 📦 Service Management

The daemon runs as a standard `systemd --user` unit:

```bash
# Check status
systemctl --user status surface-atmos

# View real-time logs
journalctl --user -u surface-atmos -f

# Restart daemon
systemctl --user restart surface-atmos
```

---

## 🗑️ Uninstallation

To cleanly remove `surface-atmos` and restore default audio settings:

```bash
cd surface-atmos
./scripts/uninstall.sh
```

---

## 📄 License

MIT License. Copyright (c) 2026 besiple.
