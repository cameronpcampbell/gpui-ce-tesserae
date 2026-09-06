The `color` benchmark measures `perceptual_feedback` and
`perceptual_brightness` through their public API. Its 64 cases cover theme
colors, saturated colors, inputs outside sRGB, near-black and near-white
colors, alternative color representations, and early returns. Inputs are
constructed before timing, and both inputs and outputs use `black_box`.

Run these commands from the workspace root. Set an explicit output directory
so saved results have the same location across runs.

```sh
export CRITERION_HOME="$PWD/target/criterion"

# Before changing the search, capture and repeat the baseline.
cargo bench --locked -p tesserae_utils --bench color -- --save-baseline before
cargo bench --locked -p tesserae_utils --bench color -- --save-baseline before-repeat

# After changing the search, compare against the repeated baseline.
cargo bench --locked -p tesserae_utils --bench color -- --baseline before-repeat
```

Each case uses a 500 ms warmup, 50 samples, and a two-second measurement
window. Keep the machine, toolchain, build flags, and power settings the same.
Run compilation and tests separately from the measurements. Inspect results
per case, since many inputs finish on the first evaluation or return before
the search.

The behavior checks run with:

```sh
cargo test --locked -p tesserae_utils -p tesserae_theme
cargo test --locked --release -p tesserae_utils --lib
```

Measurements recorded on 6 September 2026 used macOS 26.5.1 on ARM64,
Rust 1.96.0, and the default Cargo bench profile. The original search was
from commit `6b7184f`. The comparison below uses the repeated baseline,
which ran without concurrent builds. Values are Criterion's slope estimates.

| Case | Before | After | Change in time |
| --- | ---: | ---: | ---: |
| Blue feedback, +0.04 | 3.966 us | 0.753 us | -81.0% |
| Blue feedback, +0.08 | 3.350 us | 0.508 us | -84.8% |
| Blue brightness, 0.01 | 3.885 us | 0.697 us | -82.1% |
| Red brightness, 0.99 | 2.736 us | 1.095 us | -60.0% |
| Outside sRGB brightness, 0.01 | 3.633 us | 0.533 us | -85.3% |
| sRGB blue feedback, +0.04 | 4.543 us | 0.802 us | -82.3% |
| Blue brightness, 0.5 | 0.296 us | 0.294 us | -0.9% |
| Blue feedback, -0.04 | 0.295 us | 0.307 us | +4.0% |

All 19 cases that originally took more than one microsecond improved by
2.5 to 6.8 times. Some feedback cases that already finished on the first
evaluation took 1 to 4 percent longer, an increase of at most 12 ns in
the slope estimates. These results support using interpolation after the
initial candidate misses its target.

The search estimates the next progress value from the previous two
measurements. It uses bisection when the estimate falls outside the bracket
or the measurements are flat, and stops when progress cannot change. The
initial guess, 24-evaluation limit, tolerances, and GPUI display conversions
are unchanged.

A temporary diagnostic over a nine-level RGB grid and two inputs outside
sRGB measured 6,577 brightness searches and 5,188 feedback searches, excluding
endpoint returns and targets at or below `1e-7`. Mean evaluations fell from
7.36 to 2.10 for brightness and from 3.52 to 1.47 for feedback. Maximum
brightness residuals were about `2.1e-6` for both versions; maximum feedback
residuals were `4.8e-6` before and `9.2e-6` after, within the existing
relative search tolerance for those requests.

The ten behavior tests pass in debug and release builds. Their shared corpus
now includes 64 deterministic colors between grid points and perturbations
of saturated colors. The request sweeps cover tiny feedback, brightness
near the original color, and adjacent floating-point values around feedback
saturation. Alpha, gamut, hue, continuity, target accuracy, and displayed
agreement across color representations retain their existing tolerances.
