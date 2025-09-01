# 📁 folderwalk

**folderwalk** is a lightweight folder-walking tool written in Rust. It recursively scans a directory and outputs its structure—optionally including file contents. Designed for fast context generation, especially useful in AI workflows like MCP agents or file-aware editing systems.

---

## 🚀 Features

- Recursively walks a directory tree
- Outputs folder structure with optional file contents
- Supports ASCII or Unicode tree formatting
- Can limit recursion depth
- Outputs to `files.txt` or directly to stdout

---

## 🛠 Installation

Install via [Cargo](https://doc.rust-lang.org/cargo/):

```bash
cargo install folderwalk
```

---

## 📂 Usage

### Basic structure output (writes to `files.txt`):

```bash
folderwalk <path/to/folder>
```

### Include file contents:

```bash
folderwalk <path/to/folder> -c
```

### Output to stdout instead of `files.txt`:

```bash
folderwalk <path/to/folder> -o
```

### Combine options:

```bash
folderwalk <path/to/folder> -c -o --max-depth 3 --ascii
```

---

## 🔧 Options

| Flag            | Description                                      |
|-----------------|--------------------------------------------------|
| `--content`, `-c` | Include file contents in output                 |
| `--stdout`, `-o`  | Print output to stdout instead of `files.txt`  |
| `--max-depth N`   | Limit recursion to N levels                    |
| `--ascii`         | Use ASCII tree characters instead of Unicode   |
| `--help`, `-h`    | Show usage instructions                        |

---

## 📄 Output Behavior

- **Default:** Creates `files.txt` in the target directory.
- **With `-o`:** Prints to stdout instead of writing a file.
- **Excludes:** Common directories like `node_modules`, `.git`, and `target`.

---

## 💡 Use Cases

- Generate context for AI agents (e.g. MCP)
- Quickly inspect project structure
- Feed source trees into file-aware tools

---

## 🧪 Example

```bash
folderwalk ./my_project -c -o --max-depth 2
```

Outputs a tree of `./my_project` with file contents, limited to 2 levels deep, printed to stdout.

---

Made with 🦀 Rust and a dash of pragmatism.
