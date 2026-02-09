Creating database migrations is not needed. When modifying database schema
use `make reset-database` command to update `schema.rs` file.

If you run `grep` command for some reason, make sure to exclude
`target` and `.git` directories.
