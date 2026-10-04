import argparse
import base64
import io
import json
import os
import secrets
import sys
from pathlib import Path

os.environ.setdefault("HF_HUB_OFFLINE", "1")
os.environ.setdefault("TRANSFORMERS_OFFLINE", "1")


def emit(payload):
    print(json.dumps(payload, ensure_ascii=False), flush=True)


def load_pipeline(model_path):
    import torch
    from diffusers import DiffusionPipeline

    if torch.cuda.is_available():
        dtype = torch.float16
    else:
        dtype = torch.float32

    pipe = DiffusionPipeline.from_pretrained(
        str(model_path),
        torch_dtype=dtype,
        local_files_only=True,
        use_safetensors=True,
    )
    pipe.set_progress_bar_config(disable=True)
    pipe.enable_attention_slicing()

    if hasattr(pipe, "enable_vae_slicing"):
        pipe.enable_vae_slicing()

    if torch.cuda.is_available():
        # CPU offload keeps the first Create model usable on lower-VRAM GPUs.
        pipe.enable_model_cpu_offload()
        return pipe, torch, "cuda-cpu-offload", True

    pipe.to("cpu")
    return pipe, torch, "cpu", False


def clamp_int(value, low, high, fallback):
    try:
        value = int(value)
    except (TypeError, ValueError):
        return fallback
    return max(low, min(high, value))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--model-path", required=True)
    args = parser.parse_args()

    model_path = Path(args.model_path)
    if not model_path.exists():
        emit({"type": "fatal", "message": "AURA Create model path does not exist."})
        return 2

    try:
        pipe, torch, device, cuda = load_pipeline(model_path)
    except Exception as error:
        emit({"type": "fatal", "message": f"Could not load the local image model: {error}"})
        return 3

    emit({"type": "ready", "device": device, "cuda": cuda})

    for raw in sys.stdin:
        raw = raw.strip()
        if not raw:
            continue

        try:
            request = json.loads(raw)
        except Exception as error:
            emit({"type": "error", "message": f"Invalid Create request: {error}"})
            continue

        request_id = request.get("id")

        if request.get("type") == "shutdown":
            emit({"type": "shutdown", "id": request_id})
            return 0

        if request.get("type") != "generate":
            emit({
                "type": "error",
                "id": request_id,
                "message": "Unknown AURA Create runtime request.",
            })
            continue

        prompt = str(request.get("prompt") or "").strip()
        negative_prompt = str(request.get("negativePrompt") or "").strip() or None
        output_path = Path(str(request.get("outputPath") or ""))
        width = clamp_int(request.get("width"), 256, 768, 512)
        height = clamp_int(request.get("height"), 256, 768, 512)
        steps = clamp_int(request.get("steps"), 4, 40, 20)
        seed = clamp_int(request.get("seed"), 0, 2**32 - 1, secrets.randbits(32))

        if not prompt:
            emit({"type": "error", "id": request_id, "message": "Image prompt cannot be empty."})
            continue
        if width % 8 != 0 or height % 8 != 0:
            emit({
                "type": "error",
                "id": request_id,
                "message": "Image dimensions must be multiples of 8.",
            })
            continue

        try:
            output_path.parent.mkdir(parents=True, exist_ok=True)
            generator = torch.Generator(device="cpu").manual_seed(seed)

            with torch.inference_mode():
                result = pipe(
                    prompt=prompt,
                    negative_prompt=negative_prompt,
                    width=width,
                    height=height,
                    num_inference_steps=steps,
                    guidance_scale=7.5,
                    generator=generator,
                )

            image = result.images[0]
            image.save(output_path, format="PNG")

            buffer = io.BytesIO()
            image.save(buffer, format="PNG")
            data_url = "data:image/png;base64," + base64.b64encode(buffer.getvalue()).decode("ascii")

            emit({
                "type": "image",
                "id": request_id,
                "path": str(output_path),
                "dataUrl": data_url,
                "width": image.width,
                "height": image.height,
                "seed": seed,
                "device": device,
                "cuda": cuda,
            })
        except Exception as error:
            if cuda and "out of memory" in str(error).lower():
                try:
                    torch.cuda.empty_cache()
                except Exception:
                    pass
                message = (
                    "AURA Create ran out of GPU memory. Try the square preset, close other GPU-heavy "
                    "apps, or use CPU generation."
                )
            else:
                message = f"Local image generation failed: {error}"

            emit({"type": "error", "id": request_id, "message": message})


if __name__ == "__main__":
    raise SystemExit(main())
