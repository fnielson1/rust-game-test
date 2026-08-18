---
name: run-game
description: Launch this Bevy game and actually observe it — screenshot the rendered frame, drive the player with held keys, or check the wasm build in a browser. Use when asked to run, start, screenshot, or visually confirm a change in the real game rather than in tests.
---

# Running and observing rust-game-test

There is no attached display an agent can drive, so "run the game" means one of
three things. Pick by what you need to learn.

| Goal | Path | Section |
|---|---|---|
| See what the level looks like | native under Xvfb + `ffmpeg` capture | [Look at a frame](#look-at-a-frame) |
| Confirm physics/collision behaviour | native + `xdotool` held keys + a frame probe | [Drive the player](#drive-the-player) |
| Confirm the wasm build works | `trunk` + a static server + Chrome | [Check the web build](#check-the-web-build) |

Use a scratch directory (`$SP`) for logs, scripts and screenshots — never write
these into the repo.

## Look at a frame

`BEVY_ASSET_ROOT="."` is required. Without it the binary cannot find `assets/`,
and the level silently fails to load — you get an empty scene, not an error.

```bash
Xvfb :99 -screen 0 1440x900x24 & sleep 2
cd /path/to/rust-game-test
DISPLAY=:99 BEVY_ASSET_ROOT="." ./target/debug/rust-game-test > "$SP/game.log" 2>&1 &
sleep 9   # bevy needs ~8s to open the window and spawn the level
ffmpeg -y -f x11grab -video_size 1440x900 -i :99 -frames:v 1 "$SP/shot.png"
```

The game window is 1280x720 at the top-left of the 1440x900 screen; the rest of
the capture is black. It picks up the real GPU through Vulkan — no software
fallback needed.

**Read `game.log`.** `level rebuilt: despawned N, spawned M` confirms the level
loaded, and `level segment I skipped: <reason>` names any rejected segment.

Zoom into a detail rather than squinting at the full frame:

```bash
ffmpeg -y -i "$SP/shot.png" -vf "crop=200:140:660:530,scale=800:560:flags=neighbor" "$SP/zoom.png"
```

### Screen/world mapping

The camera is `ScalingMode::AutoMin { min_width: 1280, min_height: 720 }` and
follows the player, so at a 1280-wide window the mapping is 1:1 and the player
sits at the horizontal centre of the canvas. Do not trust pixel measurements for
anything quantitative — see [Measure, don't pixel-peep](#measure-dont-pixel-peep).

## Drive the player

Controls are `A` / `D` to roll and `W` to jump (`src/input_config.rs`). `A`
rolls **left**, `D` rolls **right**.

`xdotool` is not installed and there is no passwordless sudo. Unpack it locally:

```bash
mkdir -p "$SP/xdo" && cd "$SP/xdo"
apt-get download xdotool libxdo3
for d in *.deb; do dpkg-deb -x "$d" root/; done
export LD_LIBRARY_PATH=$SP/xdo/root/usr/lib/x86_64-linux-gnu
XDO=$SP/xdo/root/usr/bin/xdotool
```

**Hold the key — do not send repeated presses.** `player_input` reads
`keys.pressed(...)`. A press and release landing inside one frame leaves
`pressed()` false, so a burst of discrete keystrokes moves the ball not at all.

```bash
WIN=$(DISPLAY=:99 $XDO search --name "rust-game-test" | head -1)
DISPLAY=:99 $XDO windowactivate --sync "$WIN"
DISPLAY=:99 $XDO keydown d ; sleep 0.45 ; DISPLAY=:99 $XDO keyup d
sleep 12   # let it coast
```

Hold duration is the throttle. The ball reaches `radius * MAX_ROTATION_SPEED` =
1000 u/s, fast enough to launch off any crest (correct physics, not a bug). ~0.45s
gives a controlled ~260 u/s traverse; 5s pins it at max speed.

### Measure, don't pixel-peep

To claim anything about contact, grounding or geometry, log it. Add a temporary
system, run, analyse, then `git checkout` the files — they are otherwise
untouched, so revert is clean.

```rust
// src/player/grounded.rs — TEMPORARY, remove when done
pub fn probe_grounded(
  query: bevy::prelude::Query<(&bevy::prelude::Transform, Option<&Grounded>), With<Player>>,
) {
  for (transform, grounded) in &query {
    bevy::prelude::info!("PROBE x={:.2} y={:.2} grounded={}",
      transform.translation.x, transform.translation.y, grounded.is_some());
  }
}
```

Register with `app.add_systems(Update, probe_grounded);` in `main.rs`, then parse
`PROBE` lines out of the log and count state transitions within an x range. This
is how "the ball never loses ground contact crossing the curve" becomes a number
instead of an impression.

The same trick works for pure geometry without running at all: a `#[cfg(test)]`
probe that prints from the real types and runs under `cargo test -- --nocapture`
beats measuring pixels, and costs one build.

## Check the web build

```bash
cargo run-script build-web      # trunk build --release
```

**Trunk stages into `dist/.stage/` and only swaps into `dist/` at the very end.**
`dist/index.html` existing proves nothing about the build you just started — you
will read yesterday's artifacts. Wait on the process, not the files:

```bash
while pgrep -f "[t]runk build --release" >/dev/null; do sleep 10; done
```

Serve on 8081 **without SPA fallback**. Bevy's wasm asset reader probes for a
`<asset>.meta` sidecar and needs a real 404; an SPA fallback answers 200 with
HTML and the level fails to load. `Trunk.toml` sets `no_spa` for `trunk serve`;
a plain static server is fine because it 404s naturally:

```bash
cd dist && python3 -m http.server 8081 --bind 127.0.0.1 &
curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:8081/assets/levels/level1.level.json.meta  # must be 404
```

Then open `http://127.0.0.1:8081/` in a tab and screenshot.

**Do not try to drive the game in the browser.** Chrome throttles
`requestAnimationFrame` to ~1 fps when the window is not genuinely foreground,
even though `document.visibilityState` still reports `"visible"`. Input appears
to "stop working" when in fact the game is stepping 60x too slowly. Verify this
before blaming input:

```js
const t0 = performance.now(); let frames = 0;
await new Promise(res => { (function tick(){ frames++;
  performance.now()-t0 < 1000 ? requestAnimationFrame(tick) : res(); })(); });
frames  // ~60 healthy, ~1 throttled
```

Use the browser to confirm the wasm build *renders*; use the native path for any
interaction.

## Hot patching

Live patching only works through `cargo run-script hot` (`dx serve --hot-patch`);
a plain `cargo run` or an IDE run button can never hot patch. It runs headless
fine — pass `--interactive false` so the TUI does not demand a TTY, and give it
Xvfb for a display:

```bash
DISPLAY=:99 BEVY_ASSET_ROOT="." ~/.cargo/bin/dx serve \
  --hot-patch --features hotpatch --interactive false > "$SP/hot.log" 2>&1 &
```

`dx` logs the app's own stdout at **ERROR** level. Those lines are the game's
INFO output, not failures — read the inner level, not dx's.

Two different reload paths land in this log, and they are told apart by the
counts in `level rebuilt: despawned N, spawned M`:

- **Asset reload** (you edited `assets/levels/*.level.json`): `Reloaded
  levels/level1.level.json`, then `despawned 7, spawned 7` — the previous
  entities are cleared by `spawn_level` itself.
- **Code hot patch** (you edited a system body): `Hot-patching: ... took Nms`,
  then `despawned 0, spawned 7` — zero because `despawn_world` already cleared
  the world before `level1` re-ran.

Either way `spawned` must equal the number of valid segments in the file. A
`spawned` that grows across reloads is duplicate geometry, which is invisible on
screen because the copies stack exactly.

To prove a code patch actually reached the running app, change something you can
see — `SEGMENT_COLOR_HUE` in `src/levels/spawn_level.rs` recolours every level
surface at once. Watch for the file watcher noticing `sed -i`'s temp file: the
patch still lands, but the logged path is the temp name, not the real one.

## Gotchas

- **Foreground `sleep` is blocked** by the agent harness. Put sleeps inside a
  script and launch it with `run_in_background`, or wait with a `Monitor`
  until-loop.
- **`pkill -f rust-game-test` kills your own shell**, because the pattern matches
  the shell's own command line. Always bracket the first character:
  `pkill -f "[t]arget/debug/rust-game-test"`.
- Wait on a *fresh* condition. Waiting for a file that a previous run already
  created returns instantly and hands you a stale frame.
