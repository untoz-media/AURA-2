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
        from transformers import AutoModelForSpeechSeq2Seq, AutoProcessor
    except Exception as error:
        emit({
            "type": "fatal",
            "message": f"Speech runtime dependencies are unavailable: {error}",
        })
        return 2

    model_path = os.path.abspath(args.model_path)

    try:
        processor = AutoProcessor.from_pretrained(
            model_path,
            local_files_only=True,
            trust_remote_code=False,
        )

        cuda = bool(torch.cuda.is_available())
        device = torch.device("cuda:0" if cuda else "cpu")
        dtype = torch.float16 if cuda else torch.float32

        model = AutoModelForSpeechSeq2Seq.from_pretrained(
            model_path,
            local_files_only=True,
            trust_remote_code=False,
            torch_dtype=dtype,
            low_cpu_mem_usage=True,
        )
        model.to(device)
        model.eval()
    except Exception as error:
        traceback.print_exc(file=sys.stderr)
        emit({
            "type": "fatal",
            "message": f"Could not load the local speech-to-text model: {error}",
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

            if request_type != "transcribe":
                emit({
                    "type": "error",
                    "id": request_id,
                    "message": "Unknown speech runtime request.",
                })
                continue

            samples = request.get("samples")
            if not isinstance(samples, list):
                raise ValueError("samples must be a list")

            if not samples:
                raise ValueError("audio capture is empty")

            audio = [float(sample) for sample in samples]
            inputs = processor(
                audio,
                sampling_rate=16000,
                return_tensors="pt",
            )
            input_features = inputs.input_features.to(device=device, dtype=dtype)

            generate_kwargs = {
                "task": "transcribe",
                "max_new_tokens": 128,
            }

            with torch.inference_mode():
                generated_ids = model.generate(
                    input_features,
                    **generate_kwargs,
                )

            text = processor.batch_decode(
                generated_ids,
                skip_special_tokens=True,
            )[0].strip()

            emit({
                "type": "transcription",
                "id": request_id,
                "text": text,
            })
        except Exception as error:
            traceback.print_exc(file=sys.stderr)
            emit({
                "type": "error",
                "id": request_id,
                "message": f"Local speech-to-text failed: {error}",
            })

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
