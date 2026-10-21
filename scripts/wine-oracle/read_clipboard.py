#!/usr/bin/env python3
"""Print the X11 CLIPBOARD selection (used to capture Wine app copies)."""
import sys
import tkinter as tk

root = tk.Tk()
root.withdraw()
try:
    text = root.clipboard_get()
except tk.TclError:
    sys.exit(3)  # clipboard empty or non-text
sys.stdout.write(text)
root.destroy()
