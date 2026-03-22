# BoatVol 🎧

A lightweight Windows CLI tool that **remembers and restores volume levels** for each audio device. When you switch between speakers and Bluetooth headphones, BoatVol automatically sets the volume to your last-used level.

## Features

- 🔊 Automatically restores volume when switching audio devices
- 🦷 Works great with Bluetooth headphones that reset to 100% on connect
- 💾 Persistent per-device volume storage in `config.json`
- ⚡ Tiny footprint (~300 KB release binary)

## Installation

1. Install [Rust](https://rustup.rs/)
2. Clone and build:
   ```bash
   git clone https://github.com/YOUR_USERNAME/BoatVol.git
   cd BoatVol
   cargo build --release
