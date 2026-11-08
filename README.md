# 🎧 Surface Acoustics

**Adaptive Stage & Studio Acoustic Compensator and Posture Control for Microsoft Surface Laptop Studio on Linux.**

[![Platform](https://img.shields.io/badge/Platform-Arch%20Linux%20%7C%20Fedora%20%7C%20Ubuntu-blue.svg)](https://github.com/linux-surface/linux-surface)
[![Rust](https://img.shields.io/badge/Language-Rust%202021-orange.svg)](https://www.rust-lang.org/)
[![Audio](https://img.shields.io/badge/Audio-PipeWire%20Smart%20Filter-green.svg)](https://pipewire.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## 🌟 Overview

The **Microsoft Surface Laptop Studio (1 & 2)** features a unique hinge with three primary operating postures:
1. **Laptop Mode (`laptop`)**: Standard clamshell with exposed keyboard.
2. **Stage Mode (`slate`)**: The display pulls forward magnetically over the keyboard, resting right above the trackpad.
3. **Studio Mode (`tablet`)**: The display folds flat into a digital drawing canvas.

### The Acoustic Challenge
The Surface Laptop Studio's **Quad Omnisonic** speaker system has tweeters located directly underneath the keyboard deck, firing upward through the keycaps. 
In **Stage** and **Studio** modes, the heavy glass display physically covers the keyboard, creating a solid acoustic barrier that muffles high frequencies (dialogue, consonants, and speech clarity from 2.5 kHz to 10 kHz).

Under Windows, official Dolby Atmos drivers monitor the posture sensor and dynamically compensate for this physical obstruction.

**`surface-atmos` brings this exact adaptive acoustic behavior to Linux — transparently and with zero volume loss.**

---

## ✨ Features

- **Inline WirePlumber 0.5+ Smart Filter:** Intercepts audio directly at the hardware sink (`alsa_output.pci-0000_00_1f.3.analog-stereo`). **No duplicate virtual soundcards in GNOME Quick Settings, and no volume desynchronization bugs.**
- **Transparent Laptop Mode:** In standard Laptop mode, the filter sits at `0.0 dB` (100% bit-perfect, zero latency, full dynamic range).
- **Adaptive Stage Treble Lift (+4.5 dB):** When the display covers the keyboard in Stage mode, a high-shelf filter smoothly lifts frequencies above 3000 Hz by +4.5 dB and widens the stereo field (25%), cutting cleanly through the glass screen so speech and vocals remain crystal clear.
- **Studio Mode Tuning (+5.0 dB):** Extra high-frequency lift and 30% stereo widening when laid completely flat.
- **Glitch-Free Smooth Crossfade:** Transitions interpolate seamlessly over 200ms using smoothstep S-curves without pops or clicks.
- **Stage Trackpad Fix:** Integrates the kernel `surface_aggregator_tabletsw tablet_mode_in_slate_state=0` fix so the trackpad remains fully functional when the screen covers the keyboard.
- **Clean Bluetooth Audio:** Enforces high-fidelity stereo A2DP profiles for Bluetooth headphones, eliminating muddy 8 kHz mono "Handsfree" profiles.

---

## 🚀 Quick Installation

```bash
git clone https://github.com/besiple/surface-atmos.git
cd surface-atmos
./scripts/install.sh
```

---

## 🛠️ Usage

### Check Status
```bash
surface-atmos status
```

### Manually Test Mode
```bash
surface-atmos set laptop
surface-atmos set stage
surface-atmos set studio
```

---

## 📄 License

MIT © [besiple](https://github.com/besiple)
