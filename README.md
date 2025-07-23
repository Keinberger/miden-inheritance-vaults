# Miden Inheritance Vault Tutorial

A comprehensive tutorial for building trustless cryptocurrency inheritance vaults using Miden Notes and Miden Assembly.

## Overview

This repository contains a complete implementation of an inheritance vault system that solves one of cryptocurrency's most critical problems: the billions of dollars lost when crypto holders pass away without proper inheritance mechanisms.

**The Problem**: Traditional inheritance relies on banks, lawyers, and government institutions—exactly what cryptocurrency was designed to eliminate. Without proper mechanisms, crypto assets become permanently inaccessible when holders die.

**The Solution**: Using Miden's zero-knowledge blockchain technology, we create completely trustless inheritance vaults that operate purely through mathematics and cryptography.

### Technical Setup

- **Rust**: Follow the [Rust installation guide](https://www.rust-lang.org/tools/install)
- **Miden Node**: Set up a local Miden node using the [Miden Node Setup guide](https://0xmiden.github.io/miden-docs/imported/miden-tutorials/src/miden_node_setup.html)
- Basic familiarity with command-line tools

## Tutorial

The complete tutorial [TUTORIAL.md](TUTORIAL.md) covers everything needed to build inheritance vaults on Miden. Open the [TUTORIAL.md](TUTORIAL.md) file to get started.

## Quick Start

1. **Clone and setup**:

   ```bash
   git clone https://github.com/Keinberger/miden-inheritance-vaults
   cd miden-inheritance-vault
   ```

2. **Install dependencies**:

   ```bash
   cargo build
   ```

3. **Start your Miden node** (follow the [setup guide](https://0xmiden.github.io/miden-docs/imported/miden-tutorials/src/miden_node_setup.html))

4. **Run the tutorial**:

   ```bash
   cargo run --release
   ```

5. **Follow along** with the detailed explanations in `TUTORIAL.md`

## Additional Resources

- [Miden Documentation](https://0xmiden.github.io/miden-docs/)
- [Miden Assembly Documentation](https://0xmiden.github.io/miden-docs/glossary.html?highlight=assembly#miden-assembly)
- [Miden Notes Deep Dive](https://0xmiden.github.io/miden-docs/imported/miden-base/src/note.html)
- [Other Miden Tutorials](https://0xmiden.github.io/miden-docs/imported/miden-tutorials/)

## Impact

This tutorial demonstrates how blockchain technology can solve real-world problems affecting billions of dollars in lost cryptocurrency. By the end, you'll have built a system that:

- Operates completely trustlessly
- Requires no intermediaries
- Provides mathematical certainty
- Preserves privacy through zero-knowledge proofs
- Offers flexible control for asset owners
