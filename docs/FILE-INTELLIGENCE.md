# File Intelligence V1

AURA-2 File Intelligence provides bounded local file discovery by name and metadata.

## Goal

Help the user find files quickly without turning AURA into an unrestricted disk crawler.

V1 deliberately does **not** index or read document contents.

## Search roots

AURA resolves the user's personal directories through Tauri's platform path resolver and searches only:

- Desktop
- Documents
- Downloads
- Pictures
- Videos
- Music

The whole profile, AppData, system directories, external drives and arbitrary paths are not included.

## Safety bounds

Each search uses:

- maximum depth: 4 directories below a root
- maximum scanned entries: 8,000
- maximum retained results: 20
- no symlink traversal
- no content reads
- no automatic background indexing

If the scan hits the entry cap, the result explicitly says it may be incomplete.

## Matching

Search is case-insensitive.

Ranking order:

1. exact filename
2. filename starts with query
3. filename contains full query
4. all query tokens appear in the filename

Equal-ranked results prefer more recently modified entries.

## Metadata

For matches AURA may return:

- file/folder name
- full local path
- personal-root label
- file vs directory
- extension
- file size
- modified timestamp

No file contents are opened.

## Reveal in File Explorer

AURA can reveal a path returned by File Intelligence without executing the file.

Example:

`Reveal file "C:\\Users\\…\\Documents\\report.pdf"`

Before launching Explorer, AURA:

1. requires an absolute existing path;
2. canonicalizes the target;
3. canonicalizes the allowed personal roots;
4. verifies the target still lives under one of those roots;
5. launches File Explorer with selection only.

This prevents a symlink or manually supplied path from using the feature as an unrestricted filesystem launcher.

Opening/executing the selected file is intentionally outside the V1 boundary.

## Permissions

Filename search uses the **Read** permission class.

Reveal in Explorer uses the reversible **Act** permission class.

If Read is:

- Allow → search runs
- Ask → the normal one-shot confirmation flow is used
- Never → the search is blocked

## Commands

English:

- `Find file report`
- `Find files Artemis`
- `Search files thumbnail`
- `Look for file invoice`

Portuguese:

- `Procura ficheiro Artemis`
- `Encontra ficheiro thumbnail`
- `Pesquisa ficheiros WorldUnited`

## Privacy

AURA does not persist a filename index in V1.

Search queries/results are not added to the Beta diagnostics schema.

The result can appear in the current Chat history because that is the explicit surface where the user requested the search.
