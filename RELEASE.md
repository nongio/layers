
## Release Process

When merging to main and ready to release a new version:

**1. Ensure you're on main branch with latest changes:**
```bash
git checkout main
git pull origin main
```

**2. Generate/update CHANGELOG:**
```bash
# Preview what will be added
git cliff --unreleased

# Update CHANGELOG.md with unreleased changes
git cliff --unreleased --prepend CHANGELOG.md
```

**3. Bump version in Cargo.toml:**
```bash
# Let git-cliff suggest the next version based on commits
git cliff --bumped-version

```

**4. Update Cargo.lock:**
```bash
cargo check
```

**5. Commit the release:**
```bash
git add CHANGELOG.md Cargo.toml Cargo.lock
git commit -m "chore(release): prepare for <version>"
```

**6. Tag the release:**
```bash
git tag -a v<version> -m "Release v<version>"
```

**7. Push changes and tag:**
```bash
git push origin main
git push origin v<version>
```

**Optional:** You can combine steps 2-3 with:
```bash
git cliff --bump --unreleased --prepend CHANGELOG.md
```

This will automatically calculate the next version based on your conventional commits (following the `[bump]` config in cliff.toml).