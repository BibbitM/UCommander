# TC content plugin API (WDX) – notes

Our own summary of the parts of the TC content plugin API that UCommander
uses. It is not a copy of the guide.

Source: *Content-Plugin Guide 2.12* (2021-04-04), `contentpluginhelp212_chm.zip`
from <https://www.ghisler.com/plugins.htm>. Header version in the guide:
`contplug.h` 2.10.

## 64-bit and Unicode

- A 64-bit plugin has the extension `.wdx64`.
- In 64-bit plugins, all integer parameters stay 32-bit (`int`, `DWORD`).
  Only pointers and handles are 64-bit.
- A 64-bit plugin can be Unicode-only. TC 64-bit uses the `…W` function when
  it exists, and the ANSI function only when the `…W` function is missing.
- These functions exist only in ANSI form, also in 64-bit plugins:
  `ContentGetDetectString`, `ContentGetSupportedField`,
  `ContentGetSupportedFieldFlags`, `ContentSetDefaultParams`,
  `ContentGetDefaultSortOrder`, `ContentEditValue`, `ContentStopGetValue`,
  `ContentPluginUnloading`, `ContentGetSupportedOperators`.
- The guide contradicts itself about `ContentStopGetValue`: the Unicode page
  lists it as ANSI only, but `contplug.h` has `ContentStopGetValueW`. Check
  this before a plan point uses it.
- The guide also contradicts itself about `ContentGetValue`: `overview.htm`
  lists it as a minimum function. The manual test on TC 11.x confirmed that TC
  loads a Unicode-only 64-bit plugin.
- So field names, unit names and the detect string are ANSI.
- `ContentGetValueW` returns `ft_stringw` (not `ft_string`) for UTF-16 text.
- `ft_stringw` is not in the list of return values of
  `ContentGetSupportedField`. A text field declares `ft_string`. Then
  `ContentGetValueW` can return `ft_string` or `ft_stringw` for it. Source:
  guide page `contentgetsupportedfield.htm`, value `ft_string`. Checked
  again for point 3.

The functions use `__stdcall`. On x64 there is only one calling convention, so
Rust uses `extern "system"`.

## Functions used in P2

| Function | Signature (C) | Notes |
|---|---|---|
| `ContentGetDetectString` | `int (char* DetectString, int maxlen)` | Called at the first load. Return value is not used (return 0). `maxlen` is in bytes, currently 2 KB. |
| `ContentGetSupportedField` | `int (int FieldIndex, char* FieldName, char* Units, int maxlen)` | TC calls it with index 0, 1, 2, … until the plugin returns `ft_nomorefields`. Returns the field type. `maxlen` includes the trailing NUL, and applies to `FieldName` and to `Units`. |
| `ContentGetValueW` | `int (WCHAR* FileName, int FieldIndex, int UnitIndex, void* FieldValue, int maxlen, int flags)` | Returns the field type, or an error value. `maxlen` is in **bytes** (divide by 2 for UTF-16 characters). |
| `ContentSetDefaultParams` | `void (ContentDefaultParamStruct* dps)` | Called immediately after the DLL loads. |
| `ContentPluginUnloading` | `void (void)` | Called just before TC unloads the DLL. |

### Field and unit names

Names must not contain `.` (dot), `|` (vertical line) or `:` (colon). Spaces
are allowed. Units are one string, separated by `|`, for example
`bytes|kbytes|Mbytes`. A field without units returns an empty string, and TC
then passes `UnitIndex` 0.

### Value formats in `FieldValue`

| Type | `FieldValue` points to |
|---|---|
| `ft_numeric_32` | `i32` |
| `ft_numeric_64` | `i64` |
| `ft_numeric_floating` | `f64`, optionally followed by a NUL-terminated display string (UTF-16 in `…W`) |
| `ft_date` | 3 × `u16`: year, month, day |
| `ft_time` | 3 × `u16`: hour, minute, second (local time) |
| `ft_boolean` | `i32`. 0 is false, any other value is true. |
| `ft_string` | NUL-terminated ANSI string |
| `ft_stringw` | NUL-terminated UTF-16 string |
| `ft_multiplechoice` | NUL-terminated ANSI string (one of the unit values) |
| `ft_datetime` | `FILETIME` (UTC) |

### `flags` in `ContentGetValueW`

| Flag | Value | Meaning |
|---|---|---|
| `CONTENT_DELAYIFSLOW` | 1 | TC calls in the foreground. A slow field can return `ft_delayed`, and TC calls again in a background thread. This needs a thread-safe plugin. |
| `CONTENT_PASSTHROUGH` | 2 | Only with `contflags_passthrough_size_float`. Not used. |

### Call order and caching

The guide (page `contentgetvalue.htm`, the part about caching) says:

- TC does not mix calls for different files. It calls for the next file
  only after it is complete with the previous file. So one cache for
  each running TC is enough.
- Other calls, for other fields, can come at the same time from a
  background thread, for example for a list of search results.
- More than one TC can run at the same time.

Limit: this part of the page is in the description of full-text fields.
It is possibly only for full-text fields. The experiment below checks
the call order for other fields.

#### Experiment (point 4a)

Setup: TC 11.58, 64-bit (file version of `TOTALCMD64.EXE`), the content
plugin with a temporary log of each `ContentGetValueW` call, a folder
with 36 packages, on an SSD. The log was not committed.

| Action | Calls | Order | Thread | Flags |
|---|---|---|---|---|
| Show a column view with all 11 plugin fields. | 396 (36 files × 11 fields), in 67 ms. | File by file: all fields of a file (in field index order), then the next file. | Main thread | `CONTENT_DELAYIFSLOW` |
| Refresh the window (that view). | 36: only field 1 (**Asset class**), one call for each file. | One field for all files. | Main thread | 0 |
| Show a new column view with 4 plugin fields. | 144 (36 files × 4 fields). Only the fields of the view. | File by file. | Main thread | `CONTENT_DELAYIFSLOW` |
| Sort by different columns of that view. | None. | — | — | — |

Results:

- TC asks only for the fields of the visible columns, file by file. So
  a cache of one entry is enough for one pass: TC reads each header one
  time for all fields.
- TC sorts with the values that it already has. A sort does not call
  the plugin.
- TC asked for the files in the same order in all three passes
  (observed in the log). So a pass over a folder with more files than
  the cache has finds no file in the cache (calculated from the cache
  code, not measured).
- All calls came from the main thread. UCommander never gives
  `ft_delayed`, so TC waits for each call. About 1.9 ms for each file
  with warm file caches.
- We do not know why the refresh asked for field 1 only.

UCommander uses a cache of 16 entries that is safe for calls from more
than one thread (point 4 owner decision 7, point 4a: no change).

### `ContentDefaultParamStruct`

| Field | Type | Meaning |
|---|---|---|
| `size` | `i32` | Size of the structure in bytes. Later versions can add fields. |
| `PluginInterfaceVersionLow` | `u32` | Version digits after the point × 100 (version 2.12 → 12). |
| `PluginInterfaceVersionHi` | `u32` | Version digits before the point (2.12 → 2). |
| `DefaultIniName` | `[c_char; 260]` | Suggested INI path, in the `wincmd.ini` folder. The file is shared with other plugins, so use a unique section name. |

## Constants

Field types (return values of `ContentGetSupportedField` and
`ContentGetValueW`):

| Name | Value |
|---|---|
| `ft_nomorefields` | 0 |
| `ft_numeric_32` | 1 |
| `ft_numeric_64` | 2 |
| `ft_numeric_floating` | 3 |
| `ft_date` | 4 |
| `ft_time` | 5 |
| `ft_boolean` | 6 |
| `ft_multiplechoice` | 7 |
| `ft_string` | 8 |
| `ft_fulltext` | 9 |
| `ft_datetime` | 10 |
| `ft_stringw` | 11 |
| `ft_fulltextw` | 12 |
| `ft_comparecontent` | 100 |

Error values of `ContentGetValueW`:

| Name | Value | Meaning |
|---|---|---|
| `ft_nosuchfield` | -1 | Field index not valid. |
| `ft_fileerror` | -2 | File I/O error. |
| `ft_fieldempty` | -3 | Field is valid, but the file has no value for it. |
| `ft_ondemand` | -4 | Get the value only when the user presses Space. |
| `ft_notsupported` | -5 | Function not supported. |
| `ft_setcancel` | -6 | User cancelled the field editor. |
| `ft_delayed` | 0 | Slow field: TC asks again in the background. |

`ft_delayed` and `ft_nomorefields` have the same value (0). The meaning
depends on the function.

## Detect string

ANSI text, maximum `maxlen` bytes. A short expression that TC evaluates
before it calls the plugin. TC calls the plugin only for files that match.

- `EXT` is the file extension in uppercase. TC 9 and later accept `?` and `*`
  wildcards.
- `[n]` is byte `n` of the file (first 8192 bytes only). `SIZE` is the file
  size.
- Operators: `&`, `|`, `=`, `!=`, `<`, `>`, `( )`, `!( )`, `FIND( )`,
  `FINDI( )`. `=`/`!=`/`<`/`>` bind more strongly than `&`, and `&` more
  strongly than `|`.

UCommander P2: `EXT="UASSET" | EXT="UMAP"`.
