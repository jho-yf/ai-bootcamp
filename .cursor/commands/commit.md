# Commit Command

## Purpose
This command helps create well-structured git commits with proper formatting and conventions.

## Usage
When you want to commit code changes, use this command to:
1. Analyze the changes
2. Generate an appropriate commit message
3. Stage and commit the changes

## Commit Message Format

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `build`: Changes that affect the build system or external dependencies
- `ci`: Changes to CI configuration files and scripts
- `chore`: Other changes that don't modify src or test files
- `revert`: Reverts a previous commit

### Scope (Optional)
- `backend`: Backend/Rust changes
- `frontend`: Frontend/TypeScript changes
- `config`: Configuration changes
- `deps`: Dependency updates
- `docs`: Documentation
- `ci`: CI/CD changes

### Subject
- Use imperative, present tense: "change" not "changed" nor "changes"
- Don't capitalize the first letter
- No dot (.) at the end
- Maximum 50 characters

### Body (Optional)
- Use imperative, present tense
- Explain what and why vs. how
- Wrap at 72 characters
- Separate from subject with a blank line

### Footer (Optional)
- Reference issues: `Closes #123`, `Fixes #456`
- Breaking changes: `BREAKING CHANGE: <description>`

## Examples

### Simple commit
```
feat(backend): add ticket filtering by tags
```

### Commit with body
```
fix(frontend): resolve CORS error in development

Update vite proxy configuration to correctly forward
API requests to backend server. Also update backend
CORS settings to allow credentials.

Fixes #123
```

### Multi-scope commit
```
refactor: move pre-commit configs to project root

- Move .pre-commit-config.yaml to root
- Move .github/workflows to root
- Update all path references

Closes #456
```

### Breaking change
```
feat(backend): change API response format

BREAKING CHANGE: Ticket API now returns tags as
a nested array instead of flat structure.
Migration guide available in docs/migration.md.
```

## Workflow

1. **Check status**: Review what files have changed
   ```bash
   git status
   ```

2. **Review changes**: See what actually changed
   ```bash
   git diff
   ```

3. **Stage files**: Select files to commit
   ```bash
   git add <files>
   # or
   git add -A  # for all changes
   ```

4. **Create commit**: Use this command to generate commit message
   - Analyze the staged changes
   - Determine appropriate type and scope
   - Generate commit message following conventions
   - Execute `git commit` with the message

5. **Verify**: Check the commit was created correctly
   ```bash
   git log -1
   ```

## Rules for AI Assistant

When creating commits:

1. **Analyze changes first**: Always check `git status` and `git diff` before committing
2. **Group related changes**: Don't mix unrelated changes in one commit
3. **Use appropriate type**: Choose the most specific type that fits
4. **Be descriptive**: Write clear, concise commit messages
5. **Reference issues**: Include issue numbers when applicable
6. **Follow conventions**: Always use Conventional Commits format
7. **Ask for confirmation**: Before committing, show the proposed commit message and ask for approval

## Common Scenarios

### Scenario 1: Feature addition
```
feat(backend): implement ticket tag association

Add many-to-many relationship between tickets and tags.
Includes repository methods for tag association and
filtering tickets by tags.

Closes #42
```

### Scenario 2: Bug fix
```
fix(frontend): prevent undefined error in TicketCard

Add null check for ticket.tags before accessing .length
property. Also ensure tags array is always initialized
in ticket store.

Fixes #78
```

### Scenario 3: Configuration change
```
chore(config): move pre-commit hooks to project root

Relocate .pre-commit-config.yaml and GitHub Actions
workflows from w1-project-alpha/ to project root.
Update all path references accordingly.
```

### Scenario 4: Documentation
```
docs: add Rust best practices rules

Create comprehensive Rust best practices guide in
.cursor/rules/rust-besr-practices.mdc covering
error handling, concurrency, async patterns, and more.
```

### Scenario 5: Refactoring
```
refactor(backend): optimize repository update methods

Simplify update logic by removing redundant checks.
Improve error messages and add better type safety.
```

## Interactive Commit Process

When user requests a commit:

1. Run `git status` to see what's changed
2. Run `git diff` to see the actual changes
3. Analyze the changes and determine:
   - Type (feat, fix, refactor, etc.)
   - Scope (backend, frontend, config, etc.)
   - Subject (brief description)
   - Body (detailed explanation if needed)
   - Footer (issue references if applicable)
4. Present the proposed commit message to user
5. Ask for confirmation before executing
6. If approved, run:
   ```bash
   git commit -m "<commit message>"
   ```
7. Show the created commit with `git log -1`

## Notes

- Always respect the user's intent - don't commit without explicit request
- If changes are not staged, ask if user wants to stage them first
- For large changes, suggest breaking into multiple commits
- If commit message is rejected, ask for preferred message format
- Never force push or modify history without explicit permission
