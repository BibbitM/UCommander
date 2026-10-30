# Package format – notes

Our own notes about the Unreal Engine package format (`.uasset`, `.umap`).
Each fact has a source.

## Package file tag

| Fact | Value | Source |
|---|---|---|
| A package file starts with a 4-byte tag. | `0x9E2A83C1` | CUE4Parse (Apache-2.0), commit `21e2231`, `CUE4Parse/UE4/Objects/UObject/FPackageFileSummary.cs`: constant `PACKAGE_FILE_TAG`. |
| The tag is stored little-endian on Windows. | Bytes `C1 83 2A 9E` | Follows from the value and the byte order. To confirm with the first fixture (point 1). |
| Packages from big-endian platforms use the byte-swapped tag. | `0xC1832A9E` | CUE4Parse, same file: constant `PACKAGE_FILE_TAG_SWAPPED`. UCommander does not support these packages (P2 decision). |
