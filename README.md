# jb-vmoptions-helper

This utility scans your system environment (`$PATH`) to find every installed JetBrains IDE and manages their individual configuration files (`.vmoptions`). It allows you to set, update, or check the status of common VM settings without manually editing complex configuration files.

## Prerequisites
*   Must be run on a Unix-like system with `$PATH` configured correctly.
*   Requires JetBrains IDEs to be installed in standard directories visible via your `$PATH`.

## Installation & Running
1.  Clone the repository: `git clone [your repo url]`
2.  Compile the tool: `cargo build --release`
3.  Run the executable: `./target/release/jb-vmoptions-helper`

The helper operates in two modes, depending on how you run it:

### 1. Interactive Mode (Recommended)
Simply run the command with no arguments to open the TUI interface and manage settings visually.
```bash
./target/release/jb-vmoptions-helper
```
*   **Select Apps:** Use the list of detected IDEs on the left panel. Check or uncheck apps to select which ones you want to modify.
*   **Enter Options:** In the input area, type the option you want to set (e.g., `-Xmx4g` for 4GB memory). Press `Enter`.
*   **Apply Changes:** Click the **✔ Apply** button. The tool will write the options to each selected IDE's `.vmoptions` file.

### 2. CLI Mode (`--apply`)
Use this mode from a script or command line when you need to set an option programmatically (e.g., for continuous deployment).

**Syntax:**
```bash
./target/release/jb-vmoptions-helper --apply '<option_name>'
```
*Example:* To apply 4GB of maximum memory:
```bash
./target/release/jb-vmoptions-helper --apply '-Xmx4g'
```

## ✨ How It Works (Under the Hood)

The helper is designed for safety and accuracy by:
*   **Automatic Discovery:** Scanning your `$PATH` to find all relevant JetBrains executables.
*   **Targeting Options:** For each detected IDE, it locates or creates a corresponding `.vmoptions` file in the same directory as the binary.
*   **Safe Updates:** Before writing any option, it checks if the option already exists and prevents accidental duplication. It also creates a backup (`*.vmoptions.bak`) of the original file before modification.

## 📚 Contributing
We welcome contributions! Please check out our [CONTRIBUTING.md](./CONTRIBUTING.md) for details on how to help improve the tool.
