from setuptools import setup, find_packages

setup(
    name="indy-besu-vdr",
    version="0.0.1",
    description="Bindings Python para Indy-Besu VDR",
    packages=find_packages(where="wrappers/python"),
    package_dir={"": "wrappers/python"},
    include_package_data=True,
    package_data={
        "indy_besu_vdr": ["*.so", "*.dylib"],  # inclui Linux e macOS
    },
    python_requires=">=3.8",
    install_requires=[],
)