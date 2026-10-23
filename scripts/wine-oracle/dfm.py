#!/usr/bin/env python3
"""Extract the Delphi binary forms (TPF0) stored in a PE executable and write them as text.

The original GenMSI editor is a Delphi program: its forms (the windows and the
dialogs it shows) live inside the executable as resources, in Delphi's binary
form format. This tool reads those forms out of the file so a human can read
the original interface instead of taking screenshots of it.

The tool never runs the executable, never needs Wine and never writes to it.
It only reads the file and writes one text file per form into the output
directory given on the command line (a temporary directory by default).

Usage:
    python3 dfm.py <executable> [output-directory]
    python3 dfm.py --summary <executable>

Stdlib only, deterministic output.
"""

import argparse
import os
import re
import struct
import sys
import tempfile

# The four bytes that start every Delphi binary form.
FORM_MAGIC = b"TPF0"

# A Delphi class name looks like an identifier, optionally namespaced.
CLASS_NAME_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)*$")

# Value type codes used by Delphi's binary form format.
VT_NULL = 0
VT_LIST = 1
VT_INT8 = 2
VT_INT16 = 3
VT_INT32 = 4
VT_FLOAT = 5
VT_STRING = 6
VT_IDENT = 7
VT_FALSE = 8
VT_TRUE = 9
VT_BINARY = 10
VT_SET = 11
VT_UTF8STRING = 12
VT_DOUBLE = 13
VT_WSTRING = 16
VT_INT64 = 17
VT_LSTRING = 18
VT_ULSTRING = 20

# Strings that carry a four byte length instead of the one byte length.
FOUR_BYTE_LENGTH_TYPES = (VT_UTF8STRING, VT_WSTRING, VT_LSTRING, VT_ULSTRING)


class DfmError(Exception):
    """Raised when the bytes at an offset do not hold a well formed form."""


class Component(object):
    """One object of a form: its class, its name, its properties and its children."""

    def __init__(self, class_name, name):
        self.class_name = class_name
        self.name = name
        self.properties = []   # list of (name, value)
        self.children = []      # list of Component

    def caption(self):
        """Return the Caption of this object, or '' when it has none."""
        for name, value in self.properties:
            if name.lower() == "caption" and value[0] == "string":
                return value[1]
        return ""


class Reader(object):
    """A cursor over the bytes of the executable."""

    def __init__(self, data, pos=0):
        self.data = data
        self.pos = pos

    def byte(self):
        if self.pos >= len(self.data):
            raise DfmError("unexpected end of file at %d" % self.pos)
        value = self.data[self.pos]
        self.pos += 1
        return value

    def raw(self, count):
        if count < 0 or self.pos + count > len(self.data):
            raise DfmError("unexpected end of file at %d" % self.pos)
        chunk = self.data[self.pos:self.pos + count]
        self.pos += count
        return chunk

    def short_string(self):
        """Read a length prefixed string: one byte length, or three bytes after 255."""
        length = self.byte()
        if length == 255:
            length = int.from_bytes(self.raw(3), "little")
        return self.raw(length)

    def long_string(self):
        """Read a string that carries a four byte length."""
        length = struct.unpack("<I", self.raw(4))[0]
        return self.raw(length)


def decode_text(raw):
    """Decode form text: UTF-8 when it is valid, Windows-1252 otherwise."""
    try:
        return raw.decode("utf-8")
    except UnicodeDecodeError:
        return raw.decode("cp1252", "replace")


def read_value(reader):
    """Read one property value. Returns (kind, payload) or None for the end marker."""
    kind = reader.byte()
    if kind == VT_NULL:
        return None
    if kind == VT_LIST:
        items = []
        while True:
            item = read_value(reader)
            if item is None:
                break
            items.append(item)
        return ("list", items)
    if kind == VT_INT8:
        return ("int", struct.unpack("<b", reader.raw(1))[0])
    if kind == VT_INT16:
        return ("int", struct.unpack("<h", reader.raw(2))[0])
    if kind == VT_INT32:
        return ("int", struct.unpack("<i", reader.raw(4))[0])
    if kind == VT_FLOAT:
        return ("float", struct.unpack("<f", reader.raw(4))[0])
    if kind == VT_STRING:
        return ("string", decode_text(reader.short_string()))
    if kind == VT_IDENT:
        return ("ident", decode_text(reader.short_string()))
    if kind == VT_FALSE:
        return ("bool", False)
    if kind == VT_TRUE:
        return ("bool", True)
    if kind == VT_BINARY:
        return ("binary", reader.raw(struct.unpack("<I", reader.raw(4))[0]))
    if kind == VT_SET:
        items = []
        while True:
            item = reader.short_string()
            if not item:
                break
            items.append(decode_text(item))
        return ("set", items)
    if kind == VT_UTF8STRING:
        return ("string", decode_text(reader.long_string()))
    if kind == VT_DOUBLE:
        return ("float", struct.unpack("<d", reader.raw(8))[0])
    if kind == VT_WSTRING:
        return ("string", reader.long_string().decode("utf-16-le", "replace"))
    if kind == VT_INT64:
        return ("int", struct.unpack("<q", reader.raw(8))[0])
    if kind in (VT_LSTRING, VT_ULSTRING):
        return ("string", decode_text(reader.long_string()))
    if kind == 14:
        # A collection: a run of items, each one a set of properties.
        items = []
        while True:
            if reader.data[reader.pos] == 0:
                reader.pos += 1
                break
            if reader.byte() != 1:
                raise DfmError("bad collection item at %d" % reader.pos)
            properties = []
            while True:
                if reader.data[reader.pos] == 0:
                    reader.pos += 1
                    break
                properties.append(read_property(reader))
            items.append(properties)
        return ("collection", items)
    raise DfmError("unknown value type %d at %d" % (kind, reader.pos - 1))


def read_property(reader):
    """Read one property: its name and its value."""
    name = decode_text(reader.short_string())
    value = read_value(reader)
    if value is None:
        raise DfmError("property %r has no value at %d" % (name, reader.pos))
    return (name, value)


def read_component(reader):
    """Read one object: class name, object name, properties and children."""
    class_name = decode_text(reader.short_string())
    name = decode_text(reader.short_string())
    if not CLASS_NAME_RE.match(class_name):
        raise DfmError("bad class name %r at %d" % (class_name, reader.pos))
    component = Component(class_name, name)
    while True:
        if reader.data[reader.pos] == 0:
            reader.pos += 1
            break
        component.properties.append(read_property(reader))
    while True:
        if reader.data[reader.pos] == 0:
            reader.pos += 1
            break
        read_child_marker(reader)
        component.children.append(read_component(reader))
    return component


def read_child_marker(reader):
    """Read the optional marker that precedes a child object.

    Descendant classes carry one: 0xF1 marks an inherited child and 0xF2 is
    followed by two bytes, the last one being the position of the child.
    """
    if reader.data[reader.pos] == 0xF1:
        reader.pos += 1
    elif reader.data[reader.pos] == 0xF2:
        reader.pos += 1
        reader.raw(2)


def parse_form(data, offset):
    """Parse the form that starts at offset. Returns (component, end offset)."""
    if data[offset:offset + 4] != FORM_MAGIC:
        raise DfmError("no form magic at %d" % offset)
    reader = Reader(data, offset + 4)
    # A descendant class is announced by a marker byte after the magic.
    if reader.data[reader.pos] >= 0xF0:
        reader.pos += 1
    component = read_component(reader)
    return component, reader.pos


def format_value(value):
    """Render a value the way Delphi writes it in a text form."""
    kind, payload = value
    if kind == "string":
        return "'" + payload.replace("'", "''") + "'"
    if kind == "int":
        return str(payload)
    if kind == "float":
        return repr(payload)
    if kind == "bool":
        return "True" if payload else "False"
    if kind == "ident":
        return payload
    if kind == "set":
        return "[" + ", ".join(payload) + "]"
    if kind == "binary":
        return "{" + payload.hex().upper() + "}"
    if kind == "list":
        return "(" + ", ".join(format_value(item) for item in payload) + ")"
    raise DfmError("cannot render value of kind %s" % kind)


def write_property(lines, name, value, depth):
    """Write one property, in the layout Delphi uses for text forms."""
    pad = "  " * depth
    kind, payload = value
    if kind == "collection":
        lines.append("%s%s = <" % (pad, name))
        for item in payload:
            lines.append("%s  item" % pad)
            for item_name, item_value in item:
                write_property(lines, item_name, item_value, depth + 2)
            lines.append("%s  end" % pad)
        lines.append("%s>" % pad)
    elif kind == "list":
        lines.append("%s%s = (" % (pad, name))
        for item in payload:
            lines.append("%s  %s" % (pad, format_value(item)))
        lines.append("%s)" % pad)
    else:
        lines.append("%s%s = %s" % (pad, name, format_value(value)))


def write_component(lines, component, depth=0):
    """Write one object and everything below it."""
    pad = "  " * depth
    lines.append("%sobject %s: %s" % (pad, component.name, component.class_name))
    for name, value in component.properties:
        write_property(lines, name, value, depth + 1)
    for child in component.children:
        write_component(lines, child, depth + 1)
    lines.append("%send" % pad)


def render_form(component):
    """Render a whole form as text, in the layout of a Delphi text form."""
    lines = []
    write_component(lines, component)
    return "\n".join(lines) + "\n"


def check_pe(data):
    """Fail loudly when the file is not a PE executable."""
    if len(data) < 0x40 or data[:2] != b"MZ":
        raise DfmError("not a PE executable: the file does not start with MZ")
    pe_offset = struct.unpack_from("<I", data, 0x3C)[0]
    if pe_offset + 4 > len(data) or data[pe_offset:pe_offset + 4] != b"PE\0\0":
        raise DfmError("not a PE executable: no PE signature found")


def find_forms(data):
    """Find every form in the file. Returns (components, ignored candidates)."""
    forms = []
    ignored = 0
    offset = data.find(FORM_MAGIC)
    while offset != -1:
        try:
            component, _end = parse_form(data, offset)
        except (DfmError, IndexError, struct.error):
            component = None
        if component is None:
            ignored += 1
        else:
            forms.append((offset, component))
        offset = data.find(FORM_MAGIC, offset + 1)
    return forms, ignored


def file_name_for(component, used):
    """Build a unique file name for a form."""
    base = component.name or component.class_name
    base = re.sub(r"[^A-Za-z0-9_.-]+", "_", base)
    name = base + ".dfm"
    counter = 2
    while name.lower() in used:
        name = "%s-%d.dfm" % (base, counter)
        counter += 1
    used.add(name.lower())
    return name


def write_forms(forms, outdir):
    """Write one text file per form. Returns the list of (file name, component)."""
    used = set()
    written = []
    for _offset, component in forms:
        name = file_name_for(component, used)
        path = os.path.join(outdir, name)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(render_form(component))
        written.append((name, component))
    return written


def print_summary(forms):
    """Print one line per form: class, name and caption."""
    for _offset, component in forms:
        print("%-28s %-32s %s" % (component.class_name, component.name, component.caption()))


def main(argv=None):
    parser = argparse.ArgumentParser(
        prog="dfm.py",
        description="Extrae los formularios Delphi (TPF0) de un ejecutable PE y los escribe como texto.",
        epilog="El ejecutable no se modifica ni se ejecuta: solo se lee.",
    )
    parser.add_argument("executable", help="ejecutable PE que contiene los formularios")
    parser.add_argument("outdir", nargs="?", default=None,
                        help="directorio donde escribir los formularios (por defecto, uno temporal)")
    parser.add_argument("--summary", action="store_true",
                        help="en vez de escribir los formularios, imprime una lista con su clase, nombre y titulo")
    args = parser.parse_args(argv)

    path = args.executable
    if not os.path.isfile(path):
        print("error: %s: file not found" % path, file=sys.stderr)
        return 1
    try:
        with open(path, "rb") as handle:
            data = handle.read()
    except OSError as error:
        print("error: %s: %s" % (path, error), file=sys.stderr)
        return 1

    try:
        check_pe(data)
    except DfmError as error:
        print("error: %s: %s" % (path, error), file=sys.stderr)
        return 1

    forms, ignored = find_forms(data)
    if not forms:
        print("error: %s: no Delphi form (TPF0) found in the file" % path, file=sys.stderr)
        return 1

    if args.summary:
        print_summary(forms)
        return 0

    outdir = args.outdir
    temporary = None
    if outdir is None:
        temporary = tempfile.mkdtemp(prefix="dfm-forms-")
        outdir = temporary
    try:
        os.makedirs(outdir, exist_ok=True)
        written = write_forms(forms, outdir)
    except OSError as error:
        print("error: %s: %s" % (outdir, error), file=sys.stderr)
        return 1

    print("%d forms written to %s" % (len(written), outdir))
    if ignored:
        print("%d candidate(s) ignored: they do not hold a well formed form" % ignored)
    return 0


if __name__ == "__main__":
    sys.exit(main())
