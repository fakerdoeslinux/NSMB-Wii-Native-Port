# Port notes

What this fork changes, why, and what the numbers look like. For installing, see
[../INSTALL.md](../INSTALL.md).

---

## Changes against upstream

| File | Change |
|---|---|
| `runtime/src/hle/storage/dvd.cpp` | The BCA anti-piracy check (`DVDCheckDiskBca`) returns success, and the Linux disc read path got a RAM cache |
| `runtime/src/game_folder.cpp` | The revision gate accepts Rev 1 **and** Rev 2 (upstream accepted neither as a pair) |
| `runtime/src/main.cpp` | `display_mode` in the config actually starts the window fullscreen |
| `projects/nsmbw/tools/prelink_rels.py` | Revision-aware REL bases, read from `main.dol` (Rev 1 `0x807684C0`, Rev 2 `0x80768500`) |
| `projects/nsmbw/tools/unpack_game.py` | Accepts both revisions; a hash mismatch is a warning, not a failure |
| `projects/nsmbw/tools/nsmbw_versions.py` | Address-map path overridable with `NSMBW_ADDRESS_MAP` |
| `projects/nsmbw/tools/build-linux.sh` | Switches off SDL's X11 extensions when their headers are missing |
| `projects/nsmbw/recomp.yml` | Rev 1 DOL hash and REL load address for this dump family |
| `.github/workflows/` | Upstream CI removed; it cannot build without a game dump and failed on every push |

### The BCA check

A pressed Wii disc carries a burst-cutting-area code in a physical track that holds the game ID.
An extracted image has no such track, so `DVDCheckDiskBca` fails and the game drops to its own
fatal screen (*"An error has occurred. Press the EJECT Button…"*). Dumping the BCA from the disc
is not possible with Dolphin's extraction, so the runtime answers the check the way a valid disc
would.

The override replaces the function at translation time rather than patching generated code:

```c
PPC_NATIVE_OVERRIDE(801D0CF0, DVDCheckDiskBca_801D0CF0, int32_t, (), ());
```

It has to exist before the translator runs. Adding it to an already-translated build produces
duplicate symbols or a stale indirect-dispatch winner, which is why the fix lives in the runtime
sources instead of a patch step.

### The Linux disc read cache

Every disc read on Linux used to be an `ifstream` open + seek-to-end + seek + read + close of the
extracted file. A level load issues hundreds of them:

```
[vi] slow frame 28.8 ms (scene 0x00A): game 28.78 ... (disc 9.45 in 14 reads)
```

The Windows and Android builds already read the whole disc into memory once; Linux was excluded.
It now does the same: ~335 MiB is loaded in the background at the lowest I/O priority on a
detached thread, reads are served as a memcpy, and reads that arrive before the cache is warm use
`pread` on a descriptor kept open. The cache is skipped below 4 GiB of RAM and can be disabled
with `NSMBW_NO_DVD_PRELOAD=1`.

Measured on the same scene change:

| | before | after |
|---|---|---|
| disc time in the load frame | 9–26 ms | 1.8–2.0 ms |
| per read | ~0.37–0.67 ms | ~0.018 ms |
| worst frame entering the world map | 80.5 ms | 38.9 ms |
| frame rate in that window | 33.8 | 59–60 |

---

## Where the time goes

The port holds a locked 60 fps on an AMD Ryzen 5 3550H with integrated Vega graphics. Per-frame
accounting (`NSMBW_LOG_FPS=1`):

```
fps 60.0: game 16.66 ms (gx 2.1 ms, lock wait 0.01 ms, 2350 draws, 214 unmerged, 28 copies),
          end_frame 0.00 ms, pace 0.00 ms, worst frame 18.1 ms, jitter 0.29 ms, 0 over 20 ms
idle 7.05 ms a frame in 4.4 waits, 23% of them ending at a retrace; 1.00 retraces a frame
```

`game 16.66 ms` is the frame budget, not work. The guest thread spends ~7 ms asleep waiting for
the next vertical retrace, and of the remaining ~9.5 ms about 2 ms is graphics command encoding.
The frame rate is limited by the display, not by the port.

Sampling the guest thread (`NSMBW_PROFILE`) agrees once the sleeps are excluded: self time is flat
across thousands of translated functions, with the largest single entries being indirect
dispatch, the AX reverb HLE and GX command processing at well under 1% each.

The one genuine cost is loading. Profiling only hitching frames (`NSMBW_PROFILE_HITCH=25`) shows
the remaining ~39 ms load frames spread over a dozen unrelated functions — decompression, file
selection, mutex work — with no single hotspot to attack, which is why the fix was to remove the
I/O rather than optimise the load code.

### A note on "just port it to Rust"

Rewriting hot code in Rust would not make this faster, and no Rust was added for performance. The
numbers above show where the time goes: translated game code, flat across the whole binary, plus
~2 ms of graphics encoding. The runtime's own code is a rounding error, and the load path was
bound by I/O, which the read cache removes.

The `rust_mtx` folder under `aurora-main/lib/dolphin/mtx/` is upstream Aurora code (a matrix
library mirror). Nothing in the build compiles it.

Rust would also add a cargo step to a build that is currently plain clang + CMake, for users who
just want the game to run.

---

## Profiling flags

| Variable | Effect |
|---|---|
| `NSMBW_LOG_FPS=1` | Per-2-second frame report: fps, game/gx/disc/present split, draws, jitter, idle |
| `NSMBW_PROFILE=<start>:<length>` | 1 kHz sampling profiler for the game thread, in seconds from the first frame. Writes `profile-stacks.txt` next to `console.log` |
| `NSMBW_PROFILE_HITCH=<ms>` | Keep only samples taken while no frame was presented for that long |
| `NSMBW_PROFILE_THREAD=<name>` | Sample another thread instead (e.g. `FrameWorker`) |
| `NSMBW_NO_DVD_PRELOAD=1` | Keep the disc out of RAM |

The stacks file is readable offline with `projects/nsmbw/tools/profile-report.py` against the
unstripped executable.

Example:

```bash
NSMBW_LOG_FPS=1 NSMBW_PROFILE=5:30 ./WiiCompiled
```

## Build notes

- **Hashes.** `unpack_game.py` compares `main.dol` against the known SMNE01 hashes
  (`6d755ec2…` Rev 1, `0373b4db…` Rev 2). Dumps that differ — re-dumps, different dumping
  tools — still work; the check is a warning so a valid revision is not rejected on a hash alone.
- **Memory.** Translated shards are large; `build-linux.sh` and `rebuild-all.ps1` lower the
  parallelism on machines below 4 GB of free RAM.
- **Switching revisions or Newer SMBW.** Both translate into the same `generated/` folder, so
  switching retranslates and the first build afterwards recompiles most shards.
