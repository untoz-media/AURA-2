# AURA Create — Local Image Generation

AURA Create provides a real local image-generation path for the 0.9 Beta candidate.

## Product boundary

Image generation is implemented.

Video generation is intentionally not simulated. The Video tab remains a planned capability until a real hardware-aware backend exists.

## Model

The first local image engine uses a dedicated feature model:

- AURA model id: `create-tiny-sd`
- upstream: `segmind/tiny-sd`
- revision: `66c1a55ae6659210a4de881223ac9626df59f04c`
- license metadata: CreativeML Open RAIL-M
- approximate managed download: 1.06 GB
- role: `imageGeneration`
- not selectable as the main assistant model

AURA downloads only the model index/config/tokenizer files plus SafeTensors weights for the text encoder, UNet and VAE. The Model Manager manifest contains no `.bin` weight files.

The upstream revision is pinned instead of following `main`, so the Beta model manifest is deterministic.

## Runtime

The Managed Runtime is upgraded to runtime schema v3 and includes Diffusers.

Normal flow:

```text
Install / repair AURA Runtime
        ↓
Download AURA Create · Image
        ↓
Open Create
        ↓
Enter prompt
        ↓
Persistent local Python/Diffusers worker
        ↓
PNG saved locally + preview returned to desktop
```

The worker sets Hugging Face and Transformers offline modes before importing the inference stack and loads the pipeline with:

- `local_files_only=True`
- `use_safetensors=True`
- FP16 when CUDA is available
- FP32 on CPU
- attention slicing
- VAE slicing when supported
- model CPU offload on CUDA to reduce VRAM pressure

No AURA cloud image-generation endpoint is used.

## Generation controls

The Beta UI exposes:

- prompt, maximum 1000 characters
- optional negative prompt, maximum 1000 characters
- square: 512×512
- landscape: 640×384
- portrait: 384×640
- 4–40 inference steps
- optional 32-bit unsigned seed

A missing seed is generated locally.

## Outputs

Generated files are written to:

`Pictures/AURA Create`

when the Windows Pictures directory can be resolved. AURA Local Data is used as a fallback.

The prompt is not included in the output filename.

The desktop receives an in-memory PNG data URL for the current-session preview. The diagnostics export schema does not include generated images, prompts or arbitrary file contents.

## Lifecycle

The image worker remains resident after the first generation to avoid reloading the model for every prompt.

It is stopped when:

- the Create model is removed;
- the managed AI runtime is installed, repaired or removed;
- AURA exits normally.

If the worker exits unexpectedly, AURA kills/reaps the failed child process, marks the image runtime as Error and can create a fresh worker on the next generation request.

## Beta validation

The source validation gate syntax-checks `image_runtime.py`.

The Rust regression suite covers bounded image presets and invalid aspect-ratio rejection.

The Model Manager regression suite verifies that the Create model:

- is feature-specific and not assistant-selectable;
- uses the expected pinned revision;
- contains SafeTensors weights;
- contains no `.bin` weights;
- preserves deterministic nested Hugging Face download URLs.

Real inference still requires the Windows Beta validation pass with the managed runtime and downloaded model.
