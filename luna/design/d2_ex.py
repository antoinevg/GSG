#!/usr/bin/env python

from os import path
import subprocess
import sys
import tempfile


def has_keyword(line, keyword):
    return line.strip().startswith(keyword)


def get_value(line):
    split = line.strip().split(":")
    if len(split) != 2:
        sys.stderr.write("Warning: No value for keyword '{}'\n".format(line.strip()))
        return None
    return split[1].strip()


def get_first(file, keyword, default):
    for line in file:
        if has_keyword(line, keyword):
            value = get_value(line)
            if value is None:
                continue
            return value
    return default


def list_includes(file):
    includes = []
    for line in file:
        if has_keyword(line, "$include"):
            include = get_value(line)
            if include is None:
                continue
            includes.append(include)
    return includes


def preprocess(file):
    output = []

    for line in file:
        if has_keyword(line, "$theme"):
            continue
        elif has_keyword(line, "$layout"):
            continue
        elif has_keyword(line, "$include"):
            include = get_value(line)
            if include is None:
                continue
            include = path.join(path.dirname(file.name), include)
            if not path.exists(include):
                sys.stderr.write("Warning: Could not locate included file '{}'\n".format(include))
                continue
            with open(include) as f:
                output += preprocess(f)

        else:
            output.append(line)

    return output


def d2(input_filename, theme, layout):
    command = [
        "d2",
        f"--theme={theme}",
        f"--layout={layout}",
        input_filename,
    ]
    print("Running: \n\n  {}\n".format(" ".join(command)))
    subprocess.call(command)


# - main ----------------------------------------------------------------------

if __name__ == "__main__":
    # arguments
    # TODO flag: --quiet
    # TODO flag: --render-includes-depth or just --depth (default: 0)
    input_filename = sys.argv[1]

    # defaults
    theme = "103"
    layout = "dagre"

    # get theme
    with open(input_filename) as f:
        theme = get_first(f, "$theme", theme)

    # get layout
    with open(input_filename) as f:
        layout = get_first(f, "$layout", layout)

    # render includes
    with open(input_filename) as f:
        for include in list_includes(f):
            include = path.join(path.dirname(f.name), include)
            if not path.exists(include):
                sys.stderr.write("Warning: Could not locate included file '{}'\n".format(include))
                continue
            d2(include, theme, layout)

    # preprocess input file
    with open(input_filename) as f:
         output = preprocess(f)

    # write output to temporary file and run d2 on it
    tempdir = "/tmp"
    #tempdir = tempfile._get_default_tempdir()
    with open(path.join(tempdir, path.basename(input_filename)), "wb") as f:
    #with tempfile.NamedTemporaryFile(dir=tempdir, delete=True) as f:
        print("Writing preprocessor output to temporary file: ", f.name)
        for line in output:
            f.write(line.encode())
        f.flush()

        d2(f.name, theme, layout)
