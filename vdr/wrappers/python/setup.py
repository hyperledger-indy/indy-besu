# Copyright (c) 2024 DSR Corporation, Denver, Colorado.
# https://www.dsr-corporation.com
# SPDX-License-Identifier: Apache-2.0

"""Module setup."""

import os
import runpy
from setuptools import find_packages, setup

PACKAGE_NAME = "indy_besu_vdr"
version_meta = runpy.run_path("./{}/version.py".format(PACKAGE_NAME))
VERSION = version_meta["__version__"]

with open(os.path.abspath("./README.md"), "r") as fh:
    long_description = fh.read()

if __name__ == "__main__":
    setup(
        name=PACKAGE_NAME,
        version=VERSION,
        author="DSR Corporation",
        author_email="artem.ivanov@dsr-corporation.com",
        long_description=long_description,
        long_description_content_type="text/markdown",
        packages=find_packages(),
        include_package_data=True,
        package_data={
            "": [
                "indy_besu_vdr_uniffi.dll",
                "libindy_besu_vdr_uniffi.dylib",
                "libindy_besu_vdr_uniffi.so",
            ]
        },
        python_requires=">=3.6.3",
        classifiers=[
            "Programming Language :: Python :: 3",
            "License :: OSI Approved :: Apache Software License",
            "Operating System :: OS Independent",
        ],
    )
