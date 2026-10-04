from __future__ import annotations

import argparse
import io
import json
import os
import sys
import traceback
import wave

os.environ.setdefault("PYTHONUTF8", "1")


def emit(payload: dict) -> None:
    sys.stdout.write(json.dumps(payload, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--voice-path", required=True)
    args = parser.parse_args()

    emit({"type": "boot", "python": sys.executable})

    try:
        from piper import PiperVoice, SynthesisConfig
    except Exception as error:
        emit({
            "type": "fatal",
            "message": f"Piper TTS runtime is unavailable: {error}",
        })
        return 2

    voice_path = os.path.abspath(args.voice_path)

    try:
        voice = PiperVoice.load(voice_path, use_cuda=False)
    except Exception as error:
        traceback.print_exc(file=sys.stderr)
        emit({
            "type": "fatal",
            "message": f"Could not load the local Piper voice: {error}",
        })
        return 3

    emit({
        "type": "ready",
        "voicePath": voice_path,
        "sampleRate": int(voice.config.sample_rate),
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
                    "sampleRate": int(voice.config.sample_rate),
                })
                continue

            if request_type != "speak":
                emit({
                    "type": "error",
                    "id": request_id,
                    "message": "Unknown TTS runtime request.",
                })
                continue

            text = request.get("text")
            if not isinstance(text, str):
                raise ValueError("text must be a string")

            text = text.strip()
            if not text:
                raise ValueError("text cannot be empty")

            speed = float(request.get("speed", 1.0))
            speed = max(0.6, min(1.5, speed))

            syn_config = SynthesisConfig(
                length_scale=1.0 / speed,
            )

            with io.BytesIO() as wav_io:
                with wave.open(wav_io, "wb") as wav_file:
                    voice.synthesize_wav(text, wav_file, syn_config=syn_config)

                wav_bytes = wav_io.getvalue()

            if sys.platform != "win32":
                raise RuntimeError("In-memory speaker playback currently supports Windows only")

            import winsound

            winsound.PlaySound(
                wav_bytes,
                winsound.SND_MEMORY | winsound.SND_NODEFAULT,
            )

            emit({
                "type": "spoken",
                "id": request_id,
                "text": text,
                "bytes": len(wav_bytes),
            })
        except Exception as error:
            traceback.print_exc(file=sys.stderr)
            emit({
                "type": "error",
                "id": request_id,
                "message": f"Local text-to-speech failed: {error}",
            })

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
