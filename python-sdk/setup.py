#!/usr/bin/env python3
"""
DataFold Python SDK
Client-side key management with Ed25519 support
"""

from setuptools import setup, find_packages

# Read the README file for long description
with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

# Read requirements from requirements.txt
with open("requirements.txt", "r", encoding="utf-8") as fh:
    requirements = [line.strip() for line in fh if line.strip() and not line.startswith("#")]

# Read development requirements
with open("requirements-dev.txt", "r", encoding="utf-8") as fh:
    dev_requirements = [line.strip() for line in fh if line.strip() and not line.startswith("#")]

setup(
    name="datafold-python-sdk",
    version="0.1.0",
    author="DataFold Team",
    author_email="team@datafold.com",
    description="DataFold Python SDK for client-side key management",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/datafold/datafold",
    project_urls={
        "Bug Tracker": "https://github.com/datafold/datafold/issues",
        "Documentation": "https://github.com/datafold/datafold/tree/main/python-sdk",
        "Source Code": "https://github.com/datafold/datafold/tree/main/python-sdk",
    },
    packages=find_packages(where="src"),
    package_dir={"": "src"},
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Topic :: Security :: Cryptography",
        "Topic :: Software Development :: Libraries :: Python Modules",
    ],
    python_requires=">=3.8",
    install_requires=requirements,
    extras_require={
        "dev": dev_requirements,
        "test": ["pytest>=7.0.0", "pytest-cov>=4.0.0", "pytest-asyncio>=0.21.0"],
    },
    keywords=[
        "datafold",
        "ed25519",
        "cryptography",
        "key-management",
        "client-side",
        "security",
    ],
    include_package_data=True,
    package_data={
        "": ["*.md", "*.txt", "*.yaml", "*.yml"],
    },
    entry_points={
        "console_scripts": [
            "datafold-keygen=datafold_sdk.cli:main",
        ],
    },
)