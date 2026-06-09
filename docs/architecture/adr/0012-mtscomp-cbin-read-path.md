# ADR 0012 — Native mtscomp `.cbin` read path

**Status:** Accepted — implemented in the M0–2 `CbinReader`

## Context

The M0–2 deliverable closes with a realistic-scale, bounded-memory run: stream a real ~1-hour
Neuropixels recording in < 2 GB peak memory. The dominant source of free, hour-scale ephys data is
the International Brain Laboratory (IBL), and IBL ships its raw electrophysiology exclusively as
**mtscomp** output: a compressed binary `.cbin`, a JSON header `.ch`, and the SpikeGLX `.meta`. The
SpikeGLX (`.bin`) and Zarr readers from ADR 0010/0011 cannot read it.

This forces a path decision: (a) a native Rust `.cbin` reader, (b) a Python `mtscomp` step that
decompresses to a raw `.bin` we then stream, or (c) avoid `.cbin` by using a different, already
uncompressed long recording. Option (b) needs ~82 GB of scratch disk for one decompressed hour and
adds no engine capability; option (c) leaves Segovia unable to read the corpus practitioners actually
have. Reading the `mtscomp` format confirmed it is a near-exact fit for the existing chunk-streaming
model and small to implement.

mtscomp's format: the recording is split into ~1-second chunks; within a chunk the time-axis delta
is taken (`np.diff` along samples, the first row kept absolute), the result is serialized in the
array's `chunk_order` and `zlib`-compressed, and each chunk's byte offset is recorded in the `.ch`
JSON. Decompression is therefore per-chunk and random-access: `zlib` inflate, then a cumulative sum
along time reverses the delta. Each chunk's first row is absolute, so chunks reconstruct
independently — no cross-chunk state.

## Decision

- **Add a native `CbinReader`** implementing the ADR 0011 `ChunkSource` trait, so the compute chain
  and the Python surface gain `.cbin` input for free — a third on-disk format behind the one trait,
  alongside SpikeGLX and Zarr. The `.ch` JSON is parsed with `serde`.
- **Positioned per-chunk reads, not a whole-file mmap.** The reader holds an open `File` and reads
  only the bytes of the chunk it is about to decode (`read_exact_at` on unix, `seek_read` on
  windows). Memory-mapping the `.cbin` instead — as the SpikeGLX reader does for an *uncompressed*
  file — would let the OS page the entire compressed file into the working set as it is read through,
  so RSS would track *compressed-file size* (measured: ~1.7 GB resident for a 1.6 GB `.cbin`). That
  scales to a > 2 GB violation on a ~27 GB AP-band file and would defeat the bound the SC1 gate
  depends on. Positioned reads keep only the current compressed chunk resident, so RSS is independent
  of file size.
- **Decode per native chunk, re-chunk to the caller's size.** mtscomp's ~1-second native chunks need
  not equal the requested `chunk_samples`. The iterator reads + inflates one native chunk at a time
  (`flate2` zlib), reverses the time delta, and buffers decoded rows, emitting `chunk_samples`-row
  `Array2<i16>` slices; resident memory is bounded by `max(native_chunk, chunk_samples)` rows,
  independent of file size. The GIL is released around each chunk (`Python::detach`), as the other
  readers do.
- **Delta reversal in `i16` wrapping arithmetic.** mtscomp computes the time delta in the native
  `int16` dtype with two's-complement wraparound; the reconstruction cumulative sum uses
  `i16::wrapping_add` so recovery is exactly the inverse and bit-identical. `chunk_order` `F`
  (column-major, what IBL writes) and `C` are both honored when mapping the decoded buffer to
  `(samples, channels)`.
- **Scope to what IBL ephys actually uses, reject the rest with typed errors.** `CbinReader` accepts
  `algorithm = "zlib"`, `dtype = "int16"`, and `do_spatial_diff = false`; anything else
  (`UnsupportedAlgorithm`, `NotInt16`, `UnsupportedSpatialDiff`) is a hard error rather than a silent
  wrong read — mirroring how `ZarrReader` rejects non-`int16`. Spatial differencing, non-`int16`
  dtypes, and other algorithms are extension points to add only when a real file needs them, keeping
  the surface small and every branch validated against real data before the SC1 gate.
- **`flate2` and `serde`/`serde_json`** are the new dependencies. `flate2` defaults to the
  `miniz_oxide` pure-Rust backend, so unlike the Zarr C codecs (ADR 0011) it adds no C-linking risk to
  the wheel matrix.

## Consequences

- `(samples, channels)` `int16` is now produced by three independent on-disk formats behind one
  trait. Correctness is validated against real data: the real `Noise4Sam_g0` recording re-encoded
  through the actual `mtscomp` compressor reads back **byte-identical** to the `SpikeGlxReader`, and a
  committed tiny `mtscomp`-produced fixture (multi-native-chunk, `F` order) round-trips in the test
  suite.
- IBL's hour-scale corpus is directly streamable, which is what makes the M0–2 bounded-memory run
  possible on real data (`scripts/download_ibl_lf.py` fetches one LF-band file set via the ONE public
  API; `scripts/bench_bounded_memory.py` measures peak RSS while streaming it). Measured on a real
  46-minute, 385-channel IBL LF recording (1.6 GB `.cbin`, 5.32 GB decompressed): streamed end to end
  at ~250 MB/s in **186 MB peak RSS** — well under the 2 GB bound, and because RSS is file-size-
  independent the same bound holds for a full AP-band hour.
- The reader is intentionally partial: spatial-diff and non-`int16` `.cbin` files are refused, not
  read. If such files appear in practice, the rejection points are where support is added.
- mtscomp's `sha1_uncompressed`/`sha1_compressed` header fields are parsed-over (ignored), not
  verified per chunk; integrity relies on `zlib`'s own checks and the byte-identical cross-check in
  tests.
