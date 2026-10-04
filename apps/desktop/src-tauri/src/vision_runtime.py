from __future__ import annotations

import argparse
import json
import os
import sys
import traceback

os.environ.setdefault("TOKENIZERS_PARALLELISM", "false")


def emit(payload: dict) -> None:
    sys.stdout.write(json.dumps(payload, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model-path", required=True)
    args = parser.parse_args()

    emit({"type": "boot", "python": sys.executable})

    try:
        import torch
        from PIL import Image
        from transformers import AutoModelForMultimodalLM, AutoProcessor
    except Exception as error:
        emit({
            "type": "fatal",
            "message": f"Vision runtime dependencies are unavailable: {error}",
        })
        return 2

    model_path = os.path.abspath(args.model_path)

    try:
        cuda = bool(torch.cuda.is_available())
        dtype = torch.float16 if cuda else torch.float32

        processor = AutoProcessor.from_pretrained(
            model_path,
            local_files_only=True,
            trust_remote_code=False,
        )
        model = AutoModelForMultimodalLM.from_pretrained(
            model_path,
            local_files_only=True,
            trust_remote_code=False,
            torch_dtype=dtype,
            low_cpu_mem_usage=True,
            device_map="auto" if cuda else None,
        )
        if not cuda:
            model.to("cpu")
        model.eval()
        device = next(model.parameters()).device
    except Exception as error:
        traceback.print_exc(file=sys.stderr)
        emit({
            "type": "fatal",
            "message": f"Could not load the local Vision model: {error}",
        })
        return 3

    emit({
        "type": "ready",
        "device": str(device),
        "cuda": cuda,
    })

    for raw_line in sys.stdin:
        line = raw_line.strip()
        if not line:
            continue

        request_id = None

        try:
            request = json.loads(line)
            request_id = request.get("id")
            request_type = request.get("type")

            if request_type == "shutdown":
                emit({"type": "shutdown", "id": request_id})
                return 0

            if request_type == "ping":
                emit({
                    "type": "pong",
                    "id": request_id,
                    "device": str(device),
                    "cuda": cuda,
                })
                continue

            if request_type != "analyze":
                emit({
                    "type": "error",
                    "id": request_id,
                    "message": "Unknown Vision runtime request.",
                })
                continue

            image_path = request.get("imagePath")
            prompt = request.get("prompt")

            if not isinstance(image_path, str) or not image_path.strip():
                raise ValueError("imagePath must be a local path")
            if not isinstance(prompt, str) or not prompt.strip():
                raise ValueError("prompt must be a non-empty string")

            image_path = os.path.abspath(image_path)
            if not os.path.isfile(image_path):
                raise ValueError("Vision capture file does not exist")

            image = Image.open(image_path).convert("RGB")
            messages = [
                {
                    "role": "user",
                    "content": [
                        {"type": "image", "image": image},
                        {"type": "text", "text": prompt.strip()},
                    ],
                }
            ]

            inputs = processor.apply_chat_template(
                messages,
                add_generation_prompt=True,
                tokenize=True,
                return_dict=True,
                return_tensors="pt",
            )

            inputs = {
                key: value.to(device)
                if hasattr(value, "to")
                else value
                for key, value in inputs.items()
            }

            with torch.inference_mode():
                outputs = model.generate(
                    **inputs,
                    max_new_tokens=220,
                    do_sample=False,
                )

            prompt_length = inputs["input_ids"].shape[-1]
            generated = outputs[:, prompt_length:]
            text = processor.batch_decode(
                generated,
                skip_special_tokens=True,
            )[0].strip()

            emit({
                "type": "analysis",
                "id": request_id,
                "text": text,
            })
        except Exception as error:
            traceback.print_exc(file=sys.stderr)
            emit({
                "type": "error",
                "id": request_id,
                "message": f"Local visual understanding failed: {error}",
            })

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
