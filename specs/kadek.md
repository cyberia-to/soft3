---
title: kadek composition — the media codec stack
tags: soft3, kadek, media, codecs, video, audio, spec
status: draft
version: 0.1
---
# Kadek composition

[Kadek](https://github.com/cyberia-to/kadek) is the pure-Rust media codec
stack: video decode and encode (H.264, H.265, AV1), audio decode (FLAC,
Vorbis, MP3, AAC-LC, Opus, ALAC), containers (MP4/fMP4, Matroska/WebM,
MPEG-TS) and the shared compute plane behind them. Its
[product](https://github.com/cyberia-to/kadek/blob/master/README.md),
[specs](https://github.com/cyberia-to/kadek/tree/master/specs) and
[audits](https://github.com/cyberia-to/kadek/tree/master/audit) have one home
in that repository. Soft3 owns the composition invariants here.

This is a specification contract. Kadek is pre-implementation: specs and
interface traits exist, no codec crate decodes yet; the
[roadmap](https://github.com/cyberia-to/kadek/blob/master/README.md#roadmap)
names every phase to a working complete implementation.

## Stack obligation

The stack MUST play and record media files — video and audio particles —
on every platform cyb ships on, without a foreign decoder or encoder in
the default path. Pure Rust is the portability mechanism: the same scalar
reference implementation is the conformance oracle on every OS and
architecture, and hardware decoders are accelerators, never prerequisites.

## Owners and integration invariants

| Owner | Stack obligation |
|---|---|
| Kadek | Bit-exact decode/encode correctness, codec specs, conformance corpora, container parsers, the KernelProvider backend seam |
| Cyb (body) | Product assembly: demux → decode → texture/speaker output; A/V sync on the audio clock |
| Spark | Rendering decoded frames as textures inside the existing shader pipeline |
| Honeycrisp | Apple Silicon kernel backend (Metal/AMX) behind the KernelProvider trait |
| Radio / Tape | Qualified framing and transport of media particles; no codec authority |
| Soft3 | This composition contract; conformance evidence flows through kadek/audit into release validation |

1. Kadek owns codec truth. No component re-implements entropy decoding,
   prediction or transforms; cyb and spark consume `kadek-ifac` traits
   only. Hardware-native decode (VideoToolbox, VA-API, NVDEC, MediaCodec)
   is selected through the same trait, never reached around it.
2. `kadek-ref` — the scalar reference — is mandatory for every codec, not
   an optional fallback. A platform without hardware acceleration plays
   media through the same conforming scalar path. Bit-exactness against
   the official conformance vectors is the acceptance gate per codec.
3. One compute plane: shared kernels (bit I/O, entropy primitives,
   transforms, prediction, channel decorrelation) are implemented once in
   `kadek-ref` and reused across codecs; GPU/SIMD variants are synthesized
   per kernel, not per codec. The codec→kernel map is a kadek spec.
4. Demuxers (kadek-mp4/mkv/ts) emit packets with stream-correct pts and
   keyframe index; gapless priming/trimming (encoder delay, preskip) is
   applied at the container boundary, not guessed per codec.
5. The whole stack ships under one license — the Cyber License. No MPL,
   GPL or proprietary codec dependency enters the media path.
6. Kadek reports decode/encode capability honestly per platform: hardware
   coverage, known gaps and red gates are published evidence in
   `kadek/audit/`, never hidden behind a feature flag that silently no-ops.

Detailed codec requirements, kernel decomposition and phase gates MUST be
maintained in the kadek repository, not copied here.
