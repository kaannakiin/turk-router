# TypeScript client

Not written yet. It will mirror the Rust client in `clients/rust/turk-router`: the same module
split, the same fixture corpus under `wire/fixtures`, and a cross-language test asserting that both
produce byte-identical instruction data and account metas for the same input.

Keeping the two clients in one repository is what makes that test cheap to run.
