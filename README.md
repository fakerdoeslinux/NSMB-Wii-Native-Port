# NSMB-Wii Native PC Port (SMNE01 Rev 1 & Rev 2)

A native Linux PC port of **New Super Mario Bros. Wii (USA, SMNE01)** using the [WiiCompiled](https://github.com/WitherBlitz/NSMB-Wii-Multiplayer-Recompilation) static recompilation toolchain.

This fork adds support for both **Revision 1** and **Revision 2** of the USA dump, including patches for the revision gate, the BCA anti-piracy check, and the Rev 2 REL chain layout.

---

## Features

- 🎮 **Native Linux build** — no Dolphin or Wii required
- 🔄 **Rev 1 & Rev 2 support** — accepts both SMNE01 revision 1 and revision 2 dumps
- 🚫 **BCA check bypass** — extracted discs don't have a physical BCA track, so the anti-piracy check is HLE'd to return success
- 📦 **Rev 2 REL chain** — automatic revision detection and correct pre-linked REL layout for Rev 2's shifted heap addresses
- 🖼️ **Widescreen / Fill aspect** — 16:9 widescreen and Hor+ fill for ultrawide monitors
- 🎮 **Vulkan renderer** — powered by Aurora/Dawn with Vulkan backend
- 🎵 **Audio** — native AX/DSP voice mixing
- 📊 **MangoHud support** — run with `mangohud ./WiiCompiled` for FPS/GPU overlay

---

## What You Need

- A **legal dump** of *New Super Mario Bros. Wii* (USA, SMNE01) extracted with Dolphin's "Extract Entire Disc"
- Either **Rev 1** or **Rev 2** — both are supported
- **Linux** (tested on Fedora 44)
- Build tools: `clang`, `lld`, `cmake`, `ninja`, `llvm-ar`, `llvm-ranlib`, `.NET 8 SDK`

### Installing build dependencies (Fedora)

```bash
sudo dnf install -y clang lld alsa-lib-devel cmake ninja llvm
```

---

## Quick Start (Pre-built)

If you just want to play, download the latest release and extract your game:

1. Extract your SMNE01 disc with Dolphin:
   - Open Dolphin → right-click the game → **Properties** → **Filesystem**
   - Right-click the disc → **Extract Entire Disc** → pick a folder

2. Run the port:
   ```bash
   cd /path/to/nsmbw-local-build
   ./WiiCompiled
   ```

   On first launch, point it at your extracted disc folder (the one containing `files/` and `sys/`).

3. With MangoHud overlay (FPS + GPU/Vulkan info):
   ```bash
   mangohud ./WiiCompiled
   ```

---

## Building from Source

### 1. Clone the repository

```bash
git clone https://github.com/FAKERH0G/NSMB-Wii-Native-Port.git
cd NSMB-Wii-Native-Port
```

### 2. Install the .NET 8 SDK

```bash
# Fedora
sudo dnf install -y dotnet-sdk-8.0
```

### 3. Extract your game disc

Extract your SMNE01 dump (Rev 1 or Rev 2) with Dolphin's "Extract Entire Disc" into a folder, e.g.:

```
/home/you/nsmbw-game/DATA/
  files/
  sys/
    main.dol
    boot.bin
    fst.bin
    ...
```

### 4. Prepare the game inputs

```bash
python3 projects/nsmbw/tools/unpack_game.py /home/you/nsmbw-game/DATA ../game
python3 projects/nsmbw/tools/prelink_rels.py ../game
```

The script auto-detects the revision from `main.dol`'s hash and uses the correct REL base address:
- **Rev 1**: `0x807684C0`
- **Rev 2**: `0x80768500`

### 5. Build

```bash
./projects/nsmbw/tools/build-linux.sh --game /home/you/nsmbw-game/DATA
```

This will:
1. Build the translator
2. Translate ~40,000 functions from the DOL + REL
3. Configure CMake with clang/lld
4. Compile the native binary
5. Publish everything to `../nsmbw-local-build/`

### 6. Run

```bash
cd ../nsmbw-local-build
./WiiCompiled
```

Or with MangoHud for FPS/GPU overlay:

```bash
mangohud ./WiiCompiled
```

---

## Configuration

Edit `~/.local/share/WiiCompiled/Config.toml`:

```toml
[video]
widescreen = true          # 16:9 widescreen (true) or original 4:3 (false)
force_16_9 = false         # force 16:9 even on 4:3 surfaces
resolution_multiplier = 1.0 # render scale (1.0 = 480 lines, 0 = match screen)
show_fps = true            # in-game FPS counter
window_width = 854
window_height = 480

[paths]
dvd_root = "/home/you/nsmbw-game/DATA"
```

### Ultrawide (2560×1080, 29:1, etc.)

Set `resolution_multiplier = 0.0` and your desired window size:

```toml
resolution_multiplier = 0.0
window_width = 2560
window_height = 1080
```

The "Fill" aspect mode widens the game's 16:9 canvas to match your surface (Hor+), so ultrawide monitors get more horizontal view without stretching.

---

## What Was Patched

| File | Change |
|------|--------|
| `runtime/src/game_folder.cpp` | Revision gate accepts Rev 1 **and** Rev 2 (was Rev 1 only) |
| `runtime/src/hle/storage/dvd.cpp` | BCA anti-piracy check (`DVDCheckDiskBca`) returns success on extracted discs |
| `projects/nsmbw/tools/prelink_rels.py` | Revision-aware REL base addresses (Rev 1: `0x807684C0`, Rev 2: `0x80768500`) |
| `projects/nsmbw/tools/unpack_game.py` | Accepts Rev 1 and Rev 2 DOL hashes; non-fatal hash warning for dump variants |
| `projects/nsmbw/tools/nsmbw_versions.py` | Address map path is configurable via `NSMBW_ADDRESS_MAP` env var |
| `projects/nsmbw/tools/build-linux.sh` | Auto-disables missing SDL X11 extensions (`xfixes`, `xcursor`, `xrandr`, `xrender`, `xi`, etc.) |
| `projects/nsmbw/recomp.yml` | Rev 1/Rev 2 DOL hash and load address |
| `runtime/src/main.cpp` | Ultrawide surface size reporting |

---

## Project Structure

```
NSMB-Wii-Native-Port/
├── README.md                    # this file
├── projects/nsmbw/              # NSMBW-specific project config & tools
│   ├── recomp.yml               # translator project config (DOL hash, load address)
│   ├── BUILDING.md              # original build documentation
│   ├── MAP.txt                  # function address map
│   └── tools/
│       ├── unpack_game.py       # extracts main.dol + RELs from dumped disc
│       ├── prelink_rels.py      # pre-links the 4 boot RELs into one synthetic REL
│       ├── nsmbw_versions.py    # address-map helper
│       └── build-linux.sh       # full build script (translate → CMake → compile)
├── runtime/                     # WiiCompiled runtime (HLE, DVD, VI, audio, etc.)
│   ├── src/
│   │   ├── game_folder.cpp      # disc/revision gate
│   │   ├── hle/storage/dvd.cpp  # DVD HLE + BCA bypass
│   │   └── main.cpp             # entry point, config, window setup
│   └── include/
├── native-build/                # CMake build output (gitignored)
├── generated/                   # translated functions & build shards (gitignored)
└── nsmbw-local-build/           # published output: WiiCompiled + game/ symlink (gitignored)
```

---

## Troubleshooting

| Problem | Fix |
|---------|-----|
| *"This build runs only revision 1"* | You're running a Rev 2 dump on a Rev 1-only build. Use this fork (accepts both) or dump a Rev 1 copy. |
| *OSFatal: "Press the EJECT Button"* | BCA check failing on extracted disc — this fork bypasses it. Rebuild with the patched `dvd.cpp`. |
| *CMake error: missing X11 headers* | `build-linux.sh` auto-disables missing SDL extensions. If it persists, install `libXi-devel`, `libXfixes-devel`, etc. |
| *ld.lld not found* | Install `lld` and symlink it into `Launcher/artifacts/portable-tools/toolchain-x86_64/bin/`. |
| *Game crashes loading d_profileNP* | Wrong REL base address. Make sure `prelink_rels.py` detected your revision correctly (Rev 1 = `0x807684C0`, Rev 2 = `0x80768500`). |
| *Black screen / no window* | Make sure `dvd_root` in `Config.toml` points to your extracted `DATA/` folder. |

---

## Legal

This project is a **native port toolchain**, not a game distribution. You must provide your own legal copy of *New Super Mario Bros. Wii*. Extract it yourself with Dolphin from a disc you own.

- **Game code**: © Nintendo
- **Port toolchain**: WiiCompiled / WitherBlitz (MIT or as licensed upstream)
- **This fork's patches**: See individual files

---

## Credits

- [WitherBlitz/NSMB-Wii-Multiplayer-Recompilation](https://github.com/WitherBlitz/NSMB-Wii-Multiplayer-Recompilation) — base WiiCompiled NSMBW port
- [Aurora](https://github.com/Aurora-Wright/Aurora) — Vulkan/graphics backend
- [MangoHud](https://github.com/flatec/MangoHud) — FPS/GPU overlay
- Dolphin Emulator — disc extraction reference
