#!/usr/bin/env python3

import sys
import base64
import imghdr


def identify_file(filename):
    with open(filename, "rb") as f:
        data = f.read()

    # 1) Try to interpret the file contents as Base64-encoded data
    try:
        decoded = base64.b64decode(data, validate=True)
        # If decode succeeds, check if decoded bytes form a known image format
        img_format = imghdr.what(None, decoded)
        if img_format:
            print(
                f"File '{filename}' contains Base64-encoded {img_format.upper()} data."
            )
            return
    except (base64.binascii.Error, ValueError):
        # Not valid Base64 or decoding error
        pass

    # 2) If not Base64 or if decoding didn’t identify an image,
    #    check the raw file bytes directly
    img_format = imghdr.what(None, data)
    if img_format:
        print(f"File '{filename}' is a raw {img_format.upper()} image.")
    else:
        print(f"File '{filename}' does not appear to be a recognized image format.")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <filename>")
        sys.exit(1)

    identify_file(sys.argv[1])
