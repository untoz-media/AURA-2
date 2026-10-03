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


def build_system_prompt(model_id: str) -> str:
    identity = "AURA-1" if model_id == "aura-1" else "AURA-2"
    return (
        f"You are {identity}, the local personal computer assistant developed by Untoz. "
        "Help the user clearly, naturally and responsibly. "
        "Reply in the language the user uses. "
        "You run locally inside the AURA desktop application. "
        "Never claim that you performed a computer action unless the application explicitly "
        "provided the result of that action. Never invent access to files, apps, the internet, "
        "the screen or tools. If you lack information, say so. "
        "Be direct, useful and natural rather than robotic or overly formal."
    )


def chat_template(tokenizer, messages: list[dict[str, str]]) -> str:
    try:
        return tokenizer.apply_chat_template(
            messages,
            tokenize=False,
            add_generation_prompt=True,
            enable_thinking=False,
        )
    except TypeError:
        return tokenizer.apply_chat_template(
            messages,
            tokenize=False,
            add_generation_prompt=True,
        )


def trim_messages(
    tokenizer,
    system_prompt: str,
    conversation: list[dict[str, str]],
    desktop_context: str | None = None,
) -> list[dict[str, str]]:
    messages = [{"role": "system", "content": system_prompt}]
    if desktop_context:
        messages.append(
            {
                "role": "system",
                "content": (
                    "Current desktop context supplied by the AURA application. "
                    "Treat it as ephemeral context for this turn only and do not claim "
                    "to have inspected anything beyond these fields:\n"
                    f"{desktop_context}"
                ),
            }
        )
    messages.extend(conversation)
    budget = 2816

    while len(messages) > 2:
        text = chat_template(tokenizer, messages)
        token_count = len(tokenizer(text, add_special_tokens=False)["input_ids"])
        if token_count <= budget:
            break

        if len(messages) >= 3 and messages[1].get("role") == "user":
            del messages[1:3]
        else:
            del messages[1]

    return messages


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model-path", required=True)
    parser.add_argument("--model-id", required=True)
    args = parser.parse_args()

    emit({"type": "boot", "python": sys.executable, "modelId": args.model_id})

    try:
        import torch
        from transformers import AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig
    except Exception as error:
        emit(
            {
                "type": "fatal",
                "code": "runtime_dependencies_missing",
                "message": (
                    "The local model runtime dependencies are unavailable. "
                    "AURA requires Python with torch>=2.7, transformers>=5.0, "
                    "accelerate>=1.0 and bitsandbytes>=0.45. "
                    f"Details: {error}"
                ),
            }
        )
        return 2

    model_path = os.path.abspath(args.model_path)

    try:
        tokenizer = AutoTokenizer.from_pretrained(
            model_path,
            local_files_only=True,
            trust_remote_code=False,
        )

        load_kwargs = {
            "local_files_only": True,
            "trust_remote_code": False,
            "device_map": "auto",
        }

        if torch.cuda.is_available():
            load_kwargs["quantization_config"] = BitsAndBytesConfig(
                load_in_4bit=True,
                bnb_4bit_quant_type="nf4",
                bnb_4bit_use_double_quant=True,
                bnb_4bit_compute_dtype=torch.float16,
            )
            load_kwargs["dtype"] = torch.float16
        else:
            load_kwargs["dtype"] = "auto"

        model = AutoModelForCausalLM.from_pretrained(model_path, **load_kwargs)
        model.eval()
    except Exception as error:
        traceback.print_exc(file=sys.stderr)
        emit(
            {
                "type": "fatal",
                "code": "model_load_failed",
                "message": f"Could not load the selected local model: {error}",
            }
        )
        return 3

    device = str(getattr(model, "device", "auto"))
    emit(
        {
            "type": "ready",
            "modelId": args.model_id,
            "device": device,
            "cuda": bool(torch.cuda.is_available()),
        }
    )

    system_prompt = build_system_prompt(args.model_id)

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
                emit(
                    {
                        "type": "pong",
                        "id": request_id,
                        "modelId": args.model_id,
                        "device": device,
                    }
                )
                continue

            if request_type != "generate":
                emit(
                    {
                        "type": "error",
                        "id": request_id,
                        "message": "Unknown runtime request.",
                    }
                )
                continue

            conversation = request.get("messages")
            if not isinstance(conversation, list):
                raise ValueError("messages must be a list")

            desktop_context = request.get("context")
            if desktop_context is not None and not isinstance(desktop_context, str):
                raise ValueError("context must be a string or null")

            messages = trim_messages(
                tokenizer,
                system_prompt,
                conversation,
                desktop_context=desktop_context,
            )
            text = chat_template(tokenizer, messages)
            inputs = tokenizer(text, return_tensors="pt")
            inputs = {
                key: value.to(model.device)
                for key, value in inputs.items()
            }

            with torch.inference_mode():
                output = model.generate(
                    **inputs,
                    max_new_tokens=256,
                    do_sample=True,
                    temperature=0.7,
                    top_p=0.8,
                    repetition_penalty=1.05,
                    eos_token_id=tokenizer.eos_token_id,
                    pad_token_id=tokenizer.pad_token_id,
                )

            generated = output[0][inputs["input_ids"].shape[1] :]
            response = tokenizer.decode(
                generated,
                skip_special_tokens=True,
            ).strip()

            emit(
                {
                    "type": "response",
                    "id": request_id,
                    "text": response,
                }
            )
        except Exception as error:
            traceback.print_exc(file=sys.stderr)
            emit(
                {
                    "type": "error",
                    "id": request_id,
                    "message": f"Local generation failed: {error}",
                }
            )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
