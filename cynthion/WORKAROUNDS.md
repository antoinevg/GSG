## Luna Installation Workarounds

### lambdasoc.git's `bios` submodule changes haven't been updated to latest

    cd toolchain/

    # point to latest lambdasoc-bios
    git clone https://github.com/lambdaconcept/lambdasoc.git lambdasoc.git
    git submodule init
    git submodule update
    cd lambdasoc/software/bios
    git pull origin master
    git submodule init
    git submodule update

    point pyproject.toml to: lambdasoc = {path = "../lambdasoc.git"}

### Poetry can't resolve lambdasoc requirements: migen, litex, litedram

> Because lambdasoc (0.1.dev69+g32121f2.dirty) @
> file:///Users/antoine/GreatScott/gsg.git/luna/toolchain/lambdasoc.git
> depends on litex (*) which doesn't match any versions, lambdasoc is
> forbidden.  So, because luna depends on lambdasoc
> (0.1.dev69+g32121f2.dirty) @ ../lambdasoc.git, version
> solving failed.

    pip install "git+https://github.com/m-labs/migen@3ffd64c9b47619bd6689b44f29a8ed7c74365f14"
    pip install "git+https://github.com/enjoy-digital/litex@f9f1b8e25db6d6db1aa47a135a5f898c433d516e"
    pip install "git+https://github.com/enjoy-digital/litedram@83d18f48c7f7590096ddb35d669836d7abb3be6f"

    _or_

    add the to pyproject.toml

### "litedram does not match any versions" madness

Add `enjoy-digital/litedram` to `pyproject.toml`:

```toml
[tool.poetry.dev-dependencies]
# ...
lambdasoc = {git = "https://github.com/ktemkin/lambdasoc.git"}
litedram = {git = "https://github.com/enjoy-digital/litedram", rev = "83d18f48c7f7590096ddb35d669836d7abb3be6f"}
```

Deleting the `poetry.lock` file and re-running `poetry` should now appease poertry's version solver.



---

## Luna Usage Workarounds

### PyUSB - "No backend available"

    (gsg-luna) work:luna.git % ~/.pyenv/versions/gsg-luna/bin/apollo info
    Traceback (most recent call last):
      File "/Users/antoine/.pyenv/versions/gsg-luna/bin/apollo", line 8, in <module>    sys.exit(main())
                 ^^^^^^
      File "/Users/antoine/.pyenv/versions/3.11.1/envs/gsg-luna/lib/python3.11/site-packages/apollo_fpga/commands/cli.py", line 334, in main
        device = ApolloDebugger()
                 ^^^^^^^^^^^^^^^^
      File "/Users/antoine/.pyenv/versions/3.11.1/envs/gsg-luna/lib/python3.11/site-packages/apollo_fpga/__init__.py", line 71, in __init__
        device = usb.core.find(idVendor=vid, idProduct=pid)
                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
      File "/Users/antoine/.pyenv/versions/3.11.1/envs/gsg-luna/lib/python3.11/site-packages/usb/core.py", line 1309, in find
        raise NoBackendError('No backend available')
    usb.core.NoBackendError: No backend available

Chances are the system cannot find `libusb` - try something like:

    brew install libusb
    pip install libusb

Worst case, do something vile like:

    ln -s /opt/homebrew/lib /usr/local/lib
