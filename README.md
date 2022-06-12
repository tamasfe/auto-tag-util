# Auto Tag

A simple utility that tags releases based on package names and versions.

## Usage


```
auto-tag --commit <COMMIT_SHA> --git-user-email <GIT_USER_EMAIL> --git-user-name <GIT_USER_NAME> --dry-run
```

`auto-tag` must be run within in a git repository.

## Supported Project Files

### Cargo.toml

```toml
[package]
name = "my-lib"
version = "0.1.0"

[package.metadata.auto-tag]
enabled = true
```

The example will yield a `release-my-lib-0.1.0` tag.

### package.json

```json
{
    "name": "@myOrg/package",
    "version": "0.1.0",
    "autoTag": {
        "enabled": true
    }
}
```

The example will yield a `release-myOrg__package-0.1.0` tag.

### pyproject.toml (Poetry)

```toml
[tool.poetry]
name = "some-package"
version = "0.1.0"

[tool.auto-tag]
enabled = true
```

The example will yield a `release-some-package-0.1.0` tag.
