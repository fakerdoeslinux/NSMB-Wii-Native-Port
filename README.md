# NSMB-Wii Native PC Port (SMNE01 Rev 1 & Rev 2)

A native Linux PC port of **New Super Mario Bros. Wii (USA, SMNE01)** using the [WiiCompiled](https://github.com/WitherBlitz/NSMB-Wii-Multiplayer-Recompilation) static recompilation toolchain.

This fork adds support for both **Revision 1** and **Revision 2** of the USA dump, including patches for the revision gate, the BCA anti-piracy check, and the Rev 2 REL chain layout.

---

## Features

- 🎮 **Native Linux build** — no Dolphin or Wii required
- 🔄 **Rev 1 & Rev 2 support** — accepts both SMNE01 revision 1 and revision 2 dumps
- 🚫 **BCA check bypass** — extracted discs don't have a physical BCA track, so the anti-piracy check is HLE'd to return success
- 📦 **Rev 2 REL chain** — automatic revision detection and correct pre-linked REL layout for Rev 2's shifted heap addresses
- 🖼️ **Widescreen** — 16:9 output, resizable window, in-game FPS counter
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

The window is resizable, and the runtime writes the size you leave it at back into
`window_width`/`window_height`. `show_fps` draws the frame counter.

---

## Performance

The game holds a locked 60 fps on modest hardware (measured on an AMD Ryzen 5 3550H
with integrated Vega graphics). The interesting numbers, from the runtime's own
per-frame accounting (`NSMBW_LOG_FPS=1`):

```
fps 60.0: game 16.66 ms (gx 2.1 ms, lock wait 0.01 ms, 2350 draws, 214 unmerged, 28 copies),
          end_frame 0.00 ms, pace 0.00 ms, worst frame 18.1 ms, jitter 0.29 ms, 0 over 20 ms
idle 7.05 ms a frame in 4.4 waits, 23% of them ending at a retrace; 1.00 retraces a frame
```

`game 16.66 ms` is the frame budget, not work: the guest thread spends ~7 ms asleep
waiting for the next vertical retrace, and of the remaining ~9.5 ms about 2 ms is
graphics command encoding. There is real headroom, so the frame rate is limited by
the display, not by the port.

The one place frames did drop was **loading**: a scene change, the world map, or the
first course read from disc. Those frames were 20-90 ms because every disc read was
an `open` + `seek` + `read` + `close` of the extracted file, and a load issues
hundreds of them.

That path now matches the Windows and Android builds: on Linux the ~335 MiB of game
files are read into RAM once on a background thread at the lowest priority
(`[dvd] read cache: 1182 file(s), 335 MiB in 398 ms`), reads are served as a memcpy,
and any read that arrives before the cache is warm goes through `pread` on a
descriptor kept open. Measured on the same scene change:

| | before | after |
|---|---|---|
| disc time in the load frame | 9-26 ms | 1.8-2.0 ms |
| per read | ~0.37-0.67 ms | ~0.018 ms |
| worst frame entering the world map | 80.5 ms | 38.9 ms |
| frames per second in that window | 33.8 | 59.0-60.0 |

The cache needs 4 GiB of RAM to turn itself on; `NSMBW_NO_DVD_PRELOAD=1` disables it
anywhere (the kept-descriptor reads stay, which is still cheaper than reopening the
file).

### A note on "just port it to Rust"

Rewriting hot code in Rust would not make this faster. The measurements above show
where the time actually goes, and it is not the runtime's own code:

- ~55% of the guest thread's wall time is spent asleep in the frame-pacing wait
  (the sampler reports 70% of samples in `libc` because of it),
- of the rest, the largest single items are the translated game code itself (flat
  across thousands of functions — no hotspot), the GX command encoder (~2 ms), and
  the AX reverb HLE (~0.7%),
- loading is bound by file I/O, which the read cache now removes.

Rust would also add a cargo step to a build that is currently plain clang + CMake.
If a self-contained piece of work appears later (a mixer, a decompressor, a hasher),
it can be a small crate; there is nothing in the profile today that would benefit.

### Profiling knobs

| Variable | What it does |
|---|---|
| `NSMBW_LOG_FPS=1` | Per-2s frame report: fps, game/gx/disc/present split, draws, jitter, idle time |
| `NSMBW_PROFILE=<start>:<length>` | 1 kHz sampling profiler for the game thread; writes `profile-stacks.txt` under `~/.local/share/WiiCompiled/Logs/` |
| `NSMBW_PROFILE_HITCH=<ms>` | Keep only samples taken while no frame was presented for that long |
| `NSMBW_PROFILE_THREAD=<name>` | Sample another thread (e.g. `FrameWorker`) |
| `NSMBW_NO_DVD_PRELOAD=1` | Do not read the whole disc into RAM |

---

## What Was Patched

| File | Change |
|------|--------|
| `runtime/src/game_folder.cpp` | Revision gate accepts Rev 1 **and** Rev 2 (was Rev 1 only) |
| `runtime/src/hle/storage/dvd.cpp` | BCA anti-piracy check (`DVDCheckDiskBca`) returns success on extracted discs |
| `runtime/src/hle/storage/dvd.cpp` | Linux disc read path: RAM read cache + kept descriptors instead of open/seek/read/close per read |
| `projects/nsmbw/tools/prelink_rels.py` | Revision-aware REL base addresses (Rev 1: `0x807684C0`, Rev 2: `0x80768500`) |
| `projects/nsmbw/tools/unpack_game.py` | Accepts Rev 1 and Rev 2 DOL hashes; non-fatal hash warning for dump variants |
| `projects/nsmbw/tools/nsmbw_versions.py` | Address map path is configurable via `NSMBW_ADDRESS_MAP` env var |
| `projects/nsmbw/tools/build-linux.sh` | Auto-disables missing SDL X11 extensions (`xfixes`, `xcursor`, `xrandr`, `xrender`, `xi`, etc.) |
| `projects/nsmbw/recomp.yml` | Rev 1/Rev 2 DOL hash and load address |
| `runtime/src/main.cpp` | `display_mode` from the config starts the window fullscreen |

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
