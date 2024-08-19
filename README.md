# SerdelekDb
# Rust Database Engine

## Overview

This Rust project implements a simple database engine that supports basic SQL-like operations such as `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `CREATE`, and `DROP`. It uses Rust's powerful type system and serialization features to manage and manipulate data in a custom database format.

## Features

- **Data Management**: Create, read, update, and delete tables and records.
- **Query Language**: Basic SQL-like syntax for querying and manipulating data.
- **Serialization**: Uses `rmp_serde` for serializing and deserializing database files.
- **Error Handling**: Comprehensive error handling and reporting for various operations.

## Project Structure

- **`src/`**: Contains the main source code for the database engine.
  - **`db.rs`**: Defines the `Db` struct and its methods for managing the database.
  - **`select.rs`**: Contains functions and types related to query parsing and execution.
  - **`table.rs`**: Defines the `Table`, `Row`, and `Value` types used for managing tables and rows.
  - **`query.rs`**: Defines the `Expr` enum and associated logic for executing queries.
  - **`tokenize.rs`**: Handles tokenization of SQL-like queries.

## Getting Started

### Prerequisites

- Rust 1.70.0 or later
- Cargo (comes with Rust)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/rust-database-engine.git
   cd rust-database-engine
2 Build ther project 
'''bash cargo build
