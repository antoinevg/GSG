## Dependencies

### multimarkdown

    brew install multimarkdown

### d2

See: https://d2lang.com/tour/install

Install via brew:

    # used for rfp: HEAD-be6c3a1 (2023/01/01)
    # latest:       HEAD-b69915b (2023/01/18)
    brew install d2 --HEAD

Install via curlbash:

    curl -fsSL https://d2lang.com/install.sh | sh -s -- --dry-run
    curl -fsSL https://d2lang.com/install.sh | sh -s --

Uninstall via curlbash:

    curl -fsSL https://d2lang.com/install.sh | sh -s -- --uninstall

### watchexec

See: https://watchexec.github.io/

Install:

    cargo install watchexec-cli

Uninstall:

    cargo uninstall watchexec-cli
