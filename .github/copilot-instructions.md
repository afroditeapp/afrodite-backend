Creating database migrations is not needed. When modifying database schema
use `make reset-database` command to update `schema.rs` file.

If you run `grep` command for some reason, make sure to exclude
`target` and `.git` directories.

Don't modify `api_client` crate or code which depends on it unless explicitly
requested.
