# Commit Message Confirmation Rule

## Rule: Always Confirm Commit Messages Before Committing

Whenever code changes are ready to be committed to version control, you **MUST** write a commit message and wait for the user to confirm it before executing the commit.

## Process

1. **Complete all code changes** - Finish implementing and testing the changes
2. **Write commit message** - Draft a clear, descriptive commit message following best practices
3. **Present to user** - Show the commit message to the user for review
4. **Wait for confirmation** - Do NOT commit until the user explicitly approves
5. **Execute commit** - Only after approval, run the git commit command

## Commit Message Format

Follow conventional commit format when appropriate:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks
- `style`: Code style changes (formatting, etc.)

### Guidelines:
- Use imperative mood in subject line ("Add feature" not "Added feature")
- Keep subject line under 50 characters when possible
- Capitalize subject line
- Don't end subject line with a period
- Separate subject from body with blank line
- Wrap body at 72 characters
- Explain what and why, not how
- Reference issue numbers in footer if applicable

## Example Workflow

1. AI completes implementation
2. AI writes: "Here's the proposed commit message: [message]. Please confirm if you'd like me to proceed with this commit."
3. User reviews and either:
   - Approves: "Yes, commit it" / "Looks good" / "Approved"
   - Requests changes: "Change X to Y"
   - Rejects: "Don't commit yet"
4. AI commits only after explicit approval

## Enforcement

This rule is enforced by:
1. AI assistant (automatic - never commits without confirmation)
2. User responsibility (review all commit messages)

## Exceptions

None. This rule applies to ALL commits without exception.

---

*Last updated: When commit confirmation workflow was established*