### Documentation

All headings for all `*.md` files should start from the `###` heading variant
and not the `#` or `##` variants. Never use those variants, use the ones below
that `###`. Never use emojies within this project at all. Both in our frontend
setup or within console logs.

### Commiting

All commits must follow Conventional Commits
One conceptual change per commit.
Body is a bullet list explaining each specific change.
Do not add emojis, checkboxes, or task lists.
All commits should have their relevant scopes & bulleted listed bodies.

```bash
<type>(<scope>): <short summary>

<body (bullet list preferred)>
```

**Types**: `feat`, `fix`, `refactor`, `docs`, `style`, `test`, `chore`,
`ci`, `perf`

## Pull Requests & Github Issues

**Commit messages describe the "why," not a restatement of the diff.**
PR titles usually become commit messages, so for the title **Title**: Use
conventional commit format . Look at the recently merged PRs & git history
for examples. We prefer a concise, human readable title that explains why
the change matters.

**BAD**

> perf(platform): reduce offline notifications sync with remote db.

**GOOD**

> perf(platform): cut time taken for offline notifications to sync with
> remote database by 40%.

Open description with a simple explanation of the problem based on the dev's
original problem or solution, then briefly explain the solution. Do not lead
with an implementation inventory. **Never include a "Test plan", "Testing", or
checklist of TODOs.** PR descriptions document the change, not the
verification process. The description or summary goes at the very top of
the description as plain prose, NO heading above it, no `### Summary`, or
no `### Description`, nothing. The PR title already serves as the title;
do not repeat or re-title it. Only add ### subheadings further down if
the description genuinely has multiple sections worth separating.

Avoid using `---` within our documentation md files to split the sections.

When creating GitHub issues, or Pull Requests use clean descriptions without
checkboxes, task lists, or markdown todo items.

### Other Points to note

- Keep comments up to date! When making changes, its important to keep things in sync.
