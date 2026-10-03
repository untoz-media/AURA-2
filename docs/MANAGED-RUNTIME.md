# AURA Managed Runtime

The Managed Runtime removes the requirement for users to prepare Python, PyTorch, Transformers or BitsAndBytes manually before using local AURA models.

## User flow

The intended desktop flow is now:

1. Open **Models**
2. Click **Install runtime**
3. Wait for AURA to prepare its private AI environment
4. Download AURA-1
5. Click **Use model**
6. Open Chat
7. Start a local conversation

The AURA NSIS installer itself stays relatively small. Large Python/AI packages are installed on demand only for users who want local model inference.

## Managed location

The runtime lives under Tauri App Local Data:

`<AppLocalData>/runtime/python/`

The Python installation is private to AURA.

It does not:

- modify the user's Windows PATH
- install Python for all users
- add file associations
- install the Windows Python launcher
- add Start Menu shortcuts

## Python release

The current Windows runtime pins:

`Python 3.12.10 x86-64`

Python 3.12.10 is used because it is the last Python 3.12 maintenance release with a full official Windows installer.

Installer source:

`https://www.python.org/ftp/python/3.12.10/python-3.12.10-amd64.exe`

Expected installer size:

`26,964,224 bytes`

## Installer verification

Before execution AURA validates:

1. the exact expected installer byte size;
2. the Windows Authenticode signature using `Get-AuthenticodeSignature`.

An installer that does not return a valid Windows signature is rejected.

## Silent installation

AURA invokes the official Python installer in per-user mode:

- `/quiet`
- `InstallAllUsers=0`
- custom `TargetDir`
- `Include_pip=1`
- no launcher
- no PATH changes
- no file associations
- no shortcuts
- no docs/test suite/Tk UI

Administrator elevation is not part of the intended flow.

## PyTorch selection

AURA checks for an NVIDIA GPU with:

`nvidia-smi -L`

### NVIDIA detected

PyTorch is installed from the official CUDA 12.8 wheel index:

`https://download.pytorch.org/whl/cu128`

### No NVIDIA GPU detected

PyTorch is installed from the official CPU wheel index:

`https://download.pytorch.org/whl/cpu`

After installation AURA verifies `torch.cuda.is_available()` and records the detected CUDA device name when available.

## AI dependencies

The managed environment installs:

- PyTorch >= 2.7
- Transformers >= 5.0
- Accelerate >= 1.0
- BitsAndBytes >= 0.45
- Safetensors >= 0.4

pip itself is upgraded inside the private environment before the AI packages are installed.

## Setup phases

The Models workspace exposes live phases:

- Preparing
- Downloading Python
- Verifying installer
- Installing Python
- Preparing packages
- Installing packages
- Verifying
- Ready
- Needs repair
- Error

The Python installer download reports byte-based progress.

Package installation uses phase progress because pip does not expose a stable cross-package byte progress API.

## Verification

The final runtime verification imports:

- torch
- transformers
- accelerate
- bitsandbytes

AURA then records:

- Python version
- PyTorch version
- Transformers version
- Accelerate version
- BitsAndBytes version
- CUDA availability
- CUDA device name

Only after this verification succeeds is the environment marked **Ready**.

## Marker

Verified metadata is stored in:

`<AppLocalData>/runtime/managed-runtime.json`

If Python exists but the marker is missing or invalid, the UI reports **Needs repair** instead of silently trusting the environment.

## Repair

**Repair runtime** stops any active local model worker and recreates the managed runtime from a clean runtime directory.

Downloaded model weights are not removed because they live separately under:

`<AppLocalData>/models/`

## Removal

**Remove runtime**:

- stops the current model worker;
- deletes the managed Python/runtime directory;
- deletes the cached Python installer.

Installed AURA model weights are kept.

## Model Runtime integration

The existing inference runtime already checks:

`<AppLocalData>/runtime/python/python.exe`

When the managed environment exists it can be used on the next free-form prompt.

An explicit `AURA_PYTHON` environment override remains available for development/testing.

## Security boundary

The managed setup only downloads its pinned Python installer from the official `python.org` HTTPS origin.

PyTorch uses official PyTorch package indexes.

Other AI dependencies are installed with pip from their configured package index.

No PowerShell execution policy is changed permanently; PowerShell is only invoked non-interactively to inspect the downloaded installer's Authenticode signature.
