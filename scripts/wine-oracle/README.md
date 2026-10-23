# wine-oracle

Tools that read the original GenMSI editor (`Genmsi_20230711.backup/genmsi.exe`)
without running it. The original is a Delphi program, so its forms — the
windows and dialogs it shows — are stored inside the executable as resources,
in Delphi's binary form format (`TPF0`). These tools turn those bytes into
text, so the original interface can be read instead of photographed.

Nothing here needs Wine, and nothing here modifies the repository.

## `dfm.py` — extract the forms

```
python3 scripts/wine-oracle/dfm.py Genmsi_20230711.backup/genmsi.exe /tmp/dfm-out
```

It opens the executable, finds every `TPF0` marker, parses the form that
follows and writes one text file per form into the output directory (a
temporary directory when none is given; the path is printed on stdout).

The executable must exist and must be a PE file: otherwise the tool says so
and exits with a non-zero status. It never writes to the executable.

On the original it finds **55 forms**. One more `TPF0` marker exists in the
file, inside a bitmap resource; it does not hold a form and is ignored (the
count is reported on stdout).

### Output format

One file per form, named after the object (`FormAbout.dfm`), written in the
layout Delphi uses for a text form:

```
object FormAbout: TFormAbout
  Left = 337
  Top = 221
  BorderStyle = bsDialog
  Caption = 'Acerca de Genmsi'
  ClientHeight = 355
  ...
  object Panel1: TPanel
    Left = 0
    Top = 0
    ...
  end
end
```

Values are written the way Delphi writes them: strings between quotes, sets
between brackets, booleans as `True`/`False`, collections as `< item ... end >`
and lists as `( ... )`.

## `dfm-summary.sh` — read the list of forms

```
bash scripts/wine-oracle/dfm-summary.sh Genmsi_20230711.backup/genmsi.exe
```

It runs the extractor in summary mode and prints one line per form: its
class, its name and its caption.

```
TFormAbout                   FormAbout                      Acerca de Genmsi
TFormClassDescription        FormClassDescription           Clase de módulo
TFormReplaceClass            FormReplaceClass               Reemplazo de clase
...
```

## The binary format that is parsed

* `TPF0` marks the start of a form.
* An object is a class name and an object name, both as strings.
* A string is a length and its bytes: one byte of length, or three bytes
  (little endian) when that first byte is 255.
* Properties follow, one after another: the name of the property, then a byte
  with the type of the value, then the value. Type `0` ends the properties of
  the object; a `0` where an object would start ends it.
* The value types are: 1 list, 2 int8, 3 int16, 4 int32, 5 float, 6 string,
  7 identifier, 8 false, 9 true, 10 binary, 11 set, 12 short string, 13
  double, 14 collection, 16 wide string, 17 int64, 18 and 20 strings with a
  four byte length.
* A descendant class is announced by a marker byte: `0xF1` before the class
  name of the form, `0xF1` or `0xF2` before a child object.

The parser is checked against the PE resource table: for every form, the
bytes it reads end exactly where the resource ends.
