# Backend Rust Architecture

This document describes the modular architecture of the Pratibha Inst Rust backend.

## Project Directory Structure

We use a modular layout where each logical feature, portal, or dashboard module is isolated inside its own folder under `src/modules/`. This makes it simple to add, modify, or remove features independently without polluting the root namespace.

```text
src/
├── main.rs                   # App entrypoint (initializes DB pool, loads config, starts server)
├── config.rs                 # Strong typed environment configuration loader
├── db.rs                     # Database connection pool manager
├── errors.rs                 # Shared custom Error type (returns Express-compatible JSON error response)
├── middleware.rs             # Shared HTTP middlewares (such as JWT authentication extractor)
├── utils/                    # Shared core utility modules
│   ├── mod.rs
│   ├── jwt.rs                # JWT token signing & verification helper
│   └── password.rs           # Hashing and verification helper (bcrypt)
│
└── modules/                  # Modular features (each contains its routes, handlers, and models)
    ├── mod.rs                # Declares and groups all backend modules
    │
    ├── auth/                 # Portal: Authentication & Staff management
    │   ├── mod.rs            # Route router setup
    │   ├── handlers.rs       # Endpoints handlers
    │   └── models.rs         # Database schemas, payloads, and DTOs
    │
    ├── library/              # Portal: Library Dashboard
    │   ├── mod.rs            # Library sub-routes (books, members, issues, settings, stats)
    │   ├── handlers.rs       # Library handlers (business logic & DB queries)
    │   └── models.rs         # DTO validations & models
    │
    ├── transport/            # Portal: Transport Portal
    │   ├── mod.rs            # Transport sub-routes (vehicles, routes, vehicle expenses)
    │   ├── handlers.rs       # Transport handlers
    │   └── models.rs         # Transport models
    │
    ├── finance/              # Portal: Finance Portal
    │   ├── mod.rs            # Finance sub-routes (receipts, transactions, general expenses)
    │   ├── handlers.rs       # Finance handlers
    │   └── models.rs         # Finance models
    │
    ├── classes/              # Shared helper module: Classes CRUD
    │   ├── mod.rs            # Class routes
    │   ├── handlers.rs       # Class CRUD business logic
    │   └── models.rs         # Class DTO validation structures
    │
    └── todos/                # Shared helper module: Category-based Todos
        ├── mod.rs            # Todo routes
        ├── handlers.rs       # Todo CRUD handlers
        └── models.rs         # Todo structures
```

## Modular Guidelines

When adding or porting a new module:
1. Define the SQL tables or schemas in `models.rs`.
2. Implement validation methods on the input payloads (e.g. `RegisterPayload::validate`).
3. Write HTTP controllers / business logic inside `handlers.rs`.
4. Mount the handlers to routes in `mod.rs` using Axum routing.
5. Export the routes router from `modules/mod.rs` and nest it in `main.rs`.
