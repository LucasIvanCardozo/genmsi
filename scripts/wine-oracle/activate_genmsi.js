// KWin activation helper for the Wine oracle harness (run_oracle.sh).
// Loaded via org.kde.kwin.Scripting.loadScript + start; it activates the Wine
// virtual desktop window and prints a verifiable flag to the journal.
var want = "genmsi - Wine Desktop";
var target = null;
for (const w of workspace.windowList()) {
    if (w.caption && w.caption.indexOf(want) !== -1) {
        target = w;
        workspace.activeWindow = w;
    }
}
print("GENMSI-ACTIVE:" + (target && target.active ? "yes" : "no"));
