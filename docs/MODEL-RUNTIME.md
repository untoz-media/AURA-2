# AURA Local Model Runtime

The desktop Model Runtime connects an installed Model Manager selection to free-form AURA chat.

## Routing model

AURA keeps deterministic actions in front of the LLM.

The Core routes a message in this order:

1. deterministic Windows / computer action
2. OBS action
3. explicit memory command
4. Director Mode preset
5. other deterministic M003–M005 routes
6. **local model fallback only when no deterministic route matches**

This means prompts such as:

- `open OBS`
- `mute`
- `switch scene to Camera 2`
- `prepare match`

continue to use direct, auditable tools.

A prompt such as:

- `Explain why the Moon has phases`

falls through to the selected local model.

## Runtime compatibility

AURA-1 originally runs through:

- Python 3
- PyTorch >= 2.7
- Transformers >= 5.0
- Accelerate >= 1.0
- BitsAndBytes >= 0.45

AURA-2 preserves that runtime profile for the AURA-1 model selection instead of converting or silently replacing the upstream model.

## Persistent worker

The desktop does not spawn Python for every prompt.

On the first free-form prompt:

1. AURA resolves the verified active Model Manager installation.
2. The Rust runtime writes its embedded Python worker to App Local Data.
3. AURA locates a compatible Python command.
4. The Python worker starts once.
5. The selected model is loaded from the verified local path.
6. The worker stays resident for subsequent prompts.

This avoids reloading the multi-gigabyte model for every message.

## Offline model loading

The Python worker uses:

`local_files_only=True`

for both tokenizer and model loading.

Inference does not ask Transformers to download missing model files in the background.

If the Model Manager installation is incomplete, loading fails explicitly.

## Python discovery

AURA looks for Python in this order:

1. `AURA_PYTHON` environment variable
2. managed runtime at `<AppLocalData>/runtime/python/python.exe`
3. `python`
4. `py -3`
5. `python3`

The managed runtime can now be installed directly from the **Models** workspace. `AURA_PYTHON` remains first so developers can intentionally override the managed environment.

## Quantization

When CUDA is available, the current AURA-1 profile uses the same 4-bit strategy as AURA-1:

- BitsAndBytes 4-bit
- NF4
- double quantization
- FP16 compute
- `device_map="auto"`

If CUDA is not available, Transformers falls back to automatic local loading without the 4-bit CUDA configuration. This can be substantially slower and consume much more system memory.

## Protocol

Rust and Python communicate through newline-delimited JSON over stdin/stdout.

Startup messages:

- `boot`
- `ready`
- `fatal`

Runtime requests:

- `generate`
- `ping`
- `shutdown`

Generation responses:

- `response`
- `error`

The Python worker writes library diagnostics to stderr and reserves stdout for protocol messages.

## Conversation context

Conversation state is owned by the Rust Model Runtime.

After each successful model response AURA stores the user/assistant turn in RAM.

The current limit is 20 messages.

Old turns are removed in user/assistant pairs.

The conversation is session-local and is not automatically written to persistent memory.

## Ephemeral desktop context

M005.3 can attach the current desktop context to each free-form generation without adding it to the visible conversation history.

The runtime receives an optional per-turn context containing:

- current application
- process image
- active window title when Windows exposes one
- whether the snapshot is the real foreground window or the last external window before AURA took focus

The Python worker injects this as an ephemeral system-context message for that generation only.

The user message stored in the local conversation remains unchanged, so switching from one application/window to another does not permanently pollute the conversation history with stale desktop context.

AURA still does not inspect screen pixels or UI contents in this milestone.

## New conversation

The desktop **New conversation** button now:

- clears visible chat messages;
- clears the Rust runtime conversation context;
- keeps the already-loaded model process alive.

This gives a clean context without paying the model-loading cost again.

## Model changes

Selecting another installed model:

- updates the Model Manager selection;
- stops the currently loaded Python worker;
- clears conversation context.

The next free-form message loads the newly selected model.

Removing a model also stops the current worker.

## Application exit

The local model process is stopped explicitly when:

- the user chooses **Quit AURA** from the tray;
- the main window is closed while Background Mode is disabled.

When Background Mode is enabled, hiding the window does not unload the model.

## Chat UI

The Chat workspace now keeps a visible thread.

After the first message the initial AURA hero is replaced by:

- user messages
- AURA responses
- local loading / generating state

While the runtime is starting, the UI can display:

- Loading AURA-1…
- AURA is thinking locally…

## Runtime status

Settings → Models exposes:

- runtime state
- loaded model ID
- Python executable
- device
- CUDA availability
- last runtime error

Runtime states:

- `stopped`
- `loading`
- `ready`
- `generating`
- `error`

## Failure behavior

If no active model is selected, free-form chat fails with an explicit Models instruction.

If Python is missing, AURA reports the missing runtime rather than falling back to a cloud model.

If Python dependencies are missing, the worker reports the required AURA-1 runtime dependencies.

If model loading or generation fails, the deterministic computer/OBS layers remain available.

## Scope boundary

This runtime does not yet add:

- token streaming
- persistent conversation history
- memory retrieval injection into the LLM prompt
- screenshot / visual UI understanding
- model-driven autonomous tool selection

Those capabilities can now build on a working selected-model → local-inference path.
