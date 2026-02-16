---
name: testing
description: General testing guidance including test style, Playwright for web testing, and reliability patterns. Use when writing tests.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.3.0"
---

# Testing Skill

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

See `~/.agents/shared/safety.md` for safety rules. See `~/.agents/shared/architecture.md` for architecture principles.

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER write throwaway tests
- NEVER test implementation details

**ALWAYS** do the following:

- ALWAYS write tests before code (TDD)
- ALWAYS use descriptive test names

## Test Workflow

Copy and track progress:

```text
Test Workflow:
- [ ] Write failing test first (TDD)
- [ ] Implement minimum code to pass
- [ ] Run all tests - verify pass
- [ ] Refactor if needed
- [ ] Run tests again
- [ ] Commit
```

## Test Location

```text
project/
├── src/
└── tests/
    ├── unit/
    ├── integration/
    └── fixtures/
```

## Test Style

- **Python**: pytest functions, not unittest classes
- **JS/TS**: Vitest/Jest patterns
- **Rust**: Built-in or rstest
- **Go**: testing package

## Running Tests

- Python: `uv run pytest` or `pixi run pytest`
- JS/TS: `pnpm test`
- Rust: `cargo test`
- Go: `go test`

## Concrete Examples

### Python (pytest)

```python
def test_user_registration_fails_with_duplicate_email():
    # Arrange
    email = "exists@example.com"

    # Act & Assert
    with pytest.raises(EmailExistsError):
        register_user(email)
```

### JavaScript/TypeScript (Vitest)

```typescript
describe("UserService", () => {
  it("should throw on duplicate email", async () => {
    await expect(registerUser("exists@example.com")).rejects.toThrow("Email already exists");
  });
});
```

### Go

```go
func TestUserRegistration_DuplicateEmail(t *testing.T) {
    _, err := RegisterUser("exists@example.com")
    assert.Error(t, err)
    assert.Contains(t, err.Error(), "already exists")
}
```

## HTTP/Web Testing: Playwright

Use **Playwright** for all web interfaces (language-agnostic).

```typescript
test("homepage loads", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle(/Expected/);
});

test("form submission", async ({ page }) => {
  await page.goto("/form");
  await page.fill("#email", "test@example.com");
  await page.click("#submit");
  await expect(page.locator(".success")).toBeVisible();
});
```

## What to Test

Test every:

- API endpoint
- Web page
- Interactive feature
- Error page
- Edge case
- Validation rule

## Function Names

Descriptive: `test_user_registration_fails_with_duplicate_email()`

## No Throwaway Tests

Write real tests. Even for quick verification.

## References

See [references/TESTING_PATTERNS.md](references/TESTING_PATTERNS.md) for more patterns.
