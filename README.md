# Duplicate Finder

A simple desktop application to search for duplicate files in a specified directory. This application uses SHA-256 hashing to identify duplicates and provides a user-friendly interface with progress tracking.

## Features

- **Directory Selection**: Easily choose the directory you want to scan for duplicate files.
- **Progress Tracking**: A visual progress bar indicating the scanning progress.
- **Duplicate Detection**: Identifies files with identical content using SHA-256 hashes.
- **Cancel Scanning**: Ability to stop the scanning process at any time.

## Requirements

- **Rust**: Ensure you have the latest version of Rust installed. You can download it from [rust-lang.org](https://www.rust-lang.org/).
- **Cargo**: Cargo is the Rust package manager and is included with Rust installation.
- **Dependencies**: This application uses several external crates. They will be installed automatically with Cargo. The required crates include:
  - `eframe`: For creating the graphical user interface.
  - `egui`: For building responsive user interfaces.
  - `rfd`: For file dialog support.
  - `rayon`: For data parallelism and concurrent processing.
  - `sha2`: For computing SHA-256 hashes.
  - `walkdir`: For recursively walking through directories.
 
## Installation and Running

### Requirements

- **Rust**: Make sure you have [Rust](https://www.rust-lang.org/) installed (you can install it via [rustup](https://rustup.rs/)).

### Steps to Run

1. Clone the repository to your computer:

    ```bash
    git clone https://github.com/Junsious/DupFinder.git
    ```

2. Navigate to the project directory:

    ```bash
    cd DupFinder
    ```

3. Build and run the program:

    ```bash
    cargo run --release
    ```

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Contributors

If you would like to make changes or improvements to the project, feel free to create pull requests or open issues.

---
